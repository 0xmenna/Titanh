use crate::types::{BytesSize, Operation, OperationType};
use anyhow::Result;
use codec::{Decode, Encode};
use core::time;
use csv::Writer;
use rand::RngCore;
use std::{collections::HashMap, fs::File, path::Path, thread::sleep, time::SystemTime};
use titan_api::{
    capsules_types::{GetCapsuleOpts, PutCapsuleOpts},
    common_types::ConsistencyLevel,
    CapsulesApi, CapsulesBatch, TitanhApiBuilder,
};

pub const CHAIN_ENDPOINT: &str = "ws://127.0.0.1:9944";
pub const IPFS_ENDPOINT: &str = "http://127.0.0.1:5001";
pub const SEED_PRHASE: &str =
    "bread february program comic unveil clock output oblige jewel tell reunion hammer";

const CSV_PATHS: [(OperationType, &str); 3] = [
    (OperationType::Put, "metrics/put_metrics.csv"),
    (OperationType::Get, "metrics/get_metrics.csv"),
    (OperationType::BatchPut, "metrics/batch_metrics.csv"),
];

const ITERATIONS: u8 = 10;

pub fn contents_from_byte_range(
    start: BytesSize,
    end: BytesSize,
    samples: u64,
) -> Result<Vec<Vec<u8>>> {
    if start < end {
        return Err(anyhow::anyhow!("Start range must be less than end range"));
    }
    let chunck_size = (start - end) / samples;

    let mut rng = rand::thread_rng();
    let mut contents = Vec::new();
    for i in 0..samples {
        let size = start + (chunck_size * i);
        let mut content = vec![0u8; size.to_bytes() as usize];
        rng.fill_bytes(&mut content);
        contents.push(content);
    }

    Ok(contents)
}

pub async fn put_time<'a, Id: Encode, Value: Encode>(
    api: &'a CapsulesApi<'a>,
    id: Id,
    value: Value,
    level: ConsistencyLevel,
) -> Result<u128> {
    let mut opts = PutCapsuleOpts::default();
    opts.level = level;
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let start_time = SystemTime::now();
        api.put_with_options(&id, &value, opts.clone()).await?;
        let elapsed_time = start_time.elapsed()?;

        times.push(elapsed_time.as_millis());
        if level == ConsistencyLevel::Low {
            let duration = time::Duration::from_secs(3);
            sleep(duration);
        }
    }

    let avg = average(&times);

    Ok(avg)
}

pub async fn put_batch_time<'a, Id: Encode + Clone, Value: Encode + Clone>(
    api: &'a CapsulesApi<'a>,
    batch: CapsulesBatch<Id, Value>,
    level: ConsistencyLevel,
) -> Result<u128> {
    let mut opts = PutCapsuleOpts::default();
    opts.level = level;
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let start_time = SystemTime::now();
        api.put_batch_with_options(batch.clone(), opts.clone())
            .await?;
        let elapsed_time = start_time.elapsed()?;

        times.push(elapsed_time.as_millis());
    }

    let avg = average(&times);

    Ok(avg)
}

pub async fn get_time<'a, Id: Encode, Value: Decode>(
    api: &'a CapsulesApi<'a>,
    id: Id,
    get_opts: GetCapsuleOpts,
) -> Result<u128> {
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let start_time = SystemTime::now();
        let _: Value = api.get_with_options(&id, get_opts.clone()).await?;
        let elapsed_time = start_time.elapsed()?;

        times.push(elapsed_time.as_millis());
    }

    let avg = average(&times);

    Ok(avg)
}

fn average(times: &[u128]) -> u128 {
    let sum: u128 = times.iter().sum();
    sum / times.len() as u128
}

pub struct CsvWriter {
    writers: HashMap<OperationType, Writer<File>>,
}

impl CsvWriter {
    pub fn new() -> Self {
        let path = Path::new("metrics");
        std::fs::create_dir_all(path).unwrap();

        let mut writers = HashMap::new();
        for (op, csv_path) in CSV_PATHS.iter() {
            let file = File::create(csv_path).unwrap();
            let mut writer = Writer::from_writer(file);
            writer
                .write_record(&["size", "consistency_level", "avg_elapsed_time_ms"])
                .unwrap();
            writers.insert(*op, writer);
        }

        Self { writers }
    }

    pub fn write_metrics(
        &mut self,
        operation: Operation,
        chunck_size: u64,
        mean_time: u128,
    ) -> Result<()> {
        let level = operation.consistency_level();
        let writer = self.writers.get_mut(&operation.into()).unwrap();

        writer.serialize((chunck_size, level, mean_time))?;

        Ok(())
    }
}
