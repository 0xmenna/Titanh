use crate::types::{BytesSize, Operation, OperationType};
use anyhow::Result;
use csv::Writer;
use rand::RngCore;
use std::{collections::HashMap, fs::File, path::Path};

const CSV_PATHS: [(OperationType, &str); 3] = [
    (OperationType::Put, "data/put_metrics.csv"),
    (OperationType::Get, "data/get_metrics.csv"),
    (OperationType::BatchPut, "data/batch_metrics.csv"),
];

pub fn contents_from_byte_range(
    start: BytesSize,
    end: BytesSize,
    step: BytesSize,
) -> Result<Vec<Vec<u8>>> {
    if start > end {
        return Err(anyhow::anyhow!("Start range must be less than end range"));
    }

    let mut rng = rand::thread_rng();
    let mut contents = Vec::new();

    let mut size = start;
    while size < end {
        let mut content = vec![0u8; size.to_bytes() as usize];
        rng.fill_bytes(&mut content);
        contents.push(content);

        if size == start {
            size = step;
        } else {
            size = size + step;
        }
    }

    Ok(contents)
}

pub fn average(times: &[u128]) -> u128 {
    let sum: u128 = times.iter().sum();
    sum / times.len() as u128
}

pub struct CsvWriter {
    writers: HashMap<OperationType, Writer<File>>,
}

impl CsvWriter {
    pub fn new() -> Self {
        let path = Path::new("data");
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
        size: u64,
        mean_time: u128,
    ) -> Result<()> {
        let level = operation.consistency_level();
        let writer = self.writers.get_mut(&operation.into()).unwrap();

        writer.serialize((size, level, mean_time))?;

        Ok(())
    }
}
