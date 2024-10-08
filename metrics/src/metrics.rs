use crate::{
    types::{BytesSize, Operation},
    utils::{self, CsvWriter, CHAIN_ENDPOINT, IPFS_ENDPOINT, SEED_PRHASE},
};
use anyhow::Result;
use titan_api::{
    capsules_types::GetCapsuleOpts, common_types::ConsistencyLevel, titanh::app_registrar,
    CapsulesBatch, TitanhApi, TitanhApiBuilder,
};

pub struct MetricsController {
    pub start: BytesSize,
    pub end: BytesSize,
    pub samples: u64,

    pub api: TitanhApi,
}

impl MetricsController {
    pub async fn new(start: BytesSize, end: BytesSize, samples: u64) -> Result<Self> {
        // Get the titanh-builder api
        let api = TitanhApiBuilder::rpc(CHAIN_ENDPOINT)
            .seed(SEED_PRHASE)
            .build()
            .await?;
        Ok(Self {
            start,
            end,
            samples,
            api,
        })
    }

    pub async fn compute_metrics(self) -> Result<()> {
        let mut csv_writer = CsvWriter::new();

        let app_registrar = self.api.app_registrar();
        let (app, _) = app_registrar.create_app().await?;

        let capsules = self.api.capsules().config(IPFS_ENDPOINT, app)?;
        let contents = utils::contents_from_byte_range(self.start, self.end, self.samples)?;

        for (i, content) in contents.iter().enumerate() {
            let size = content.len() as u64;
            for level in ConsistencyLevel::iter() {
                let mut id = String::new();
                id.push_str(&i.to_string());
                id.push_str(&level.to_string());
                let put_time = utils::put_time(&capsules, id, content, level).await?;
                csv_writer.write_metrics(Operation::Put(level), size, put_time)?;
            }
        }

        for (i, content) in contents.iter().enumerate() {
            let size = content.len() as u64;

            for level in GetCapsuleOpts::iter() {
                let mut id = String::new();
                id.push_str(&i.to_string());
                let consistency_level = ConsistencyLevel::from(level);
                id.push_str(&consistency_level.to_string());
                let get_time =
                    utils::get_time::<String, Vec<u8>>(&capsules, id, level.clone()).await?;
                csv_writer.write_metrics(Operation::Get(level), size, get_time)?;
            }
        }

        // Batch put metrics
        let mut batch = CapsulesBatch::new();
        let mut batch_size = 0;
        for (i, content) in contents.iter().enumerate() {
            let id = i + 1000;
            batch.insert((i as u32, content));
            batch_size += content.len() as u64;
        }

        for level in ConsistencyLevel::iter() {
            let batch_put_time = utils::put_batch_time(&capsules, batch.clone(), level).await?;
            csv_writer.write_metrics(Operation::BatchPut(level), batch_size, batch_put_time)?;
        }

        Ok(())
    }
}
