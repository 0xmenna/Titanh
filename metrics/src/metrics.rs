use crate::{
    types::{BytesSize, Operation},
    utils::{self, CsvWriter},
};
use anyhow::Result;
use rand::{rngs::ThreadRng, Rng};
use std::{
    thread::sleep,
    time::{self, SystemTime},
};
use titan_api::{
    capsules_types::{GetCapsuleOpts, PutCapsuleOpts},
    common_types::ConsistencyLevel,
    CapsulesApi, CapsulesBatch, TitanhApi, TitanhApiBuilder,
};

pub const CHAIN_ENDPOINT: &str = "ws://127.0.0.1:9944";
pub const IPFS_ENDPOINT: &str = "http://127.0.0.1:5001";
pub const SEED_PRHASE: &str =
    "bread february program comic unveil clock output oblige jewel tell reunion hammer";

const ITERATIONS: u8 = 1;

pub struct MetricsController {
    start: BytesSize,
    end: BytesSize,
    step: BytesSize,

    // Titanh api
    api: TitanhApi,
}

impl MetricsController {
    pub async fn new(start: BytesSize, end: BytesSize, step: BytesSize) -> Result<Self> {
        // Get the titanh-builder api
        let api = TitanhApiBuilder::rpc(CHAIN_ENDPOINT)
            .seed(SEED_PRHASE)
            .build()
            .await?;

        Ok(Self {
            start,
            end,
            step,
            api,
        })
    }

    pub async fn compute_metrics(self) -> Result<()> {
        let mut csv_writer = CsvWriter::new();

        let app_registrar = self.api.app_registrar();
        println!("Creating app for capsules upload..");
        let (app, _) = app_registrar.create_app().await?;

        let capsules = self.api.capsules().config(IPFS_ENDPOINT, app)?;
        // Contents to upload
        println!("Generating contents to upload..");
        let contents = utils::contents_from_byte_range(self.start, self.end, self.step)?;

        let mut rng = rand::thread_rng();
        let mut ids = Vec::new();

        for content in contents.iter() {
            let size = content.len() as u64;
            for level in ConsistencyLevel::iter() {
                println!(
                    "Put content of size: {} bytes, with consistency level: {:?}",
                    size, level
                );
                println!("...");
                let put_time = self
                    .put_time(&mut rng, &mut ids, &capsules, content, level)
                    .await?;
                csv_writer.write_metrics(Operation::Put(level), size, put_time)?;
            }
        }

        for (id, size) in ids.iter() {
            for get_opts in GetCapsuleOpts::iter() {
                println!(
                    "Get content of size: {} bytes, with get options: {:?}",
                    size, get_opts
                );
                let get_time = self.get_time(&capsules, *id, get_opts).await?;
                csv_writer.write_metrics(Operation::Get(get_opts.clone()), *size, get_time)?;
            }
        }

        // Send batch put operations
        for level in ConsistencyLevel::iter() {
            println!("Batch put with consistency level: {:?}", level);
            let batch_put_time = self
                .put_batch_time(&mut rng, &capsules, &contents, level)
                .await?;

            let batch_size = contents.iter().map(|content| content.len() as u64).sum();
            csv_writer.write_metrics(Operation::BatchPut(level), batch_size, batch_put_time)?;
        }

        Ok(())
    }

    pub async fn put_time<'a>(
        &self,
        rng: &mut ThreadRng,
        ids: &mut Vec<(u64, u64)>,
        api: &'a CapsulesApi<'a>,
        value: &Vec<u8>,
        level: ConsistencyLevel,
    ) -> Result<u128> {
        let mut opts = PutCapsuleOpts::default();
        opts.level = level;

        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let id = rng.gen::<u64>();

            let start_time = SystemTime::now();
            api.put_with_options(&id, &value, opts.clone()).await?;
            let elapsed_time = start_time.elapsed()?;

            times.push(elapsed_time.as_millis());

            ids.push((id, value.len() as u64));

            let duration = time::Duration::from_secs(10);
            sleep(duration);
        }

        let avg = utils::average(&times);

        Ok(avg)
    }

    pub async fn get_time<'a>(
        &self,
        api: &'a CapsulesApi<'a>,
        id: u64,
        get_opts: GetCapsuleOpts,
    ) -> Result<u128> {
        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let start_time = SystemTime::now();
            let _: u64 = api.get_with_options(&id, get_opts.clone()).await?;
            let elapsed_time = start_time.elapsed()?;

            times.push(elapsed_time.as_millis());
        }

        let avg = utils::average(&times);

        Ok(avg)
    }

    pub async fn put_batch_time<'a>(
        &self,
        rng: &mut ThreadRng,
        api: &'a CapsulesApi<'a>,
        contents: &Vec<Vec<u8>>,
        level: ConsistencyLevel,
    ) -> Result<u128> {
        let mut opts = PutCapsuleOpts::default();
        opts.level = level;
        let mut times = Vec::new();

        for _ in 0..ITERATIONS {
            let mut batch = CapsulesBatch::new();
            for value in contents.iter() {
                let id = rng.gen::<u64>();
                batch.insert((id, value));
            }

            let start_time = SystemTime::now();
            api.put_batch_with_options(batch.clone(), opts.clone())
                .await?;
            let elapsed_time = start_time.elapsed()?;

            times.push(elapsed_time.as_millis());

            let duration = time::Duration::from_secs(10);
            sleep(duration);
        }

        let avg = utils::average(&times);

        Ok(avg)
    }
}
