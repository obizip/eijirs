use anyhow::{Context, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use sled::Batch;
use std::fs::File;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::Path;

const BATCH_SIZE: usize = 10_000;

#[derive(Debug)]
pub struct Db {
    db: sled::Db,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }

    pub fn init(db_path: &Path, dict_path: &Path) -> Result<Self> {
        let db: sled::Db = sled::open(db_path).context("Failed to open database")?;
        let dict = File::open(dict_path)?;

        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(encoding_rs::SHIFT_JIS))
                .build(dict),
        );

        let mut count = 0;
        let mut batch = Batch::default();

        for line in reader.lines() {
            let line = line?;
            if !line.starts_with('■') {
                continue;
            }
            // "■見出し : 訳語" の分割
            if let Some((key, value)) = line.split_once(" : ") {
                let key = key.trim_start_matches("■").trim();
                let value = value.trim();

                batch.insert(key, value);
                count += 1;

                if count % BATCH_SIZE == 0 {
                    db.apply_batch(batch)?;
                    batch = Batch::default();
                    print!("\rProcessed {} entries...", count);
                    io::stdout().flush()?;
                }
            }
        }
        db.apply_batch(batch)?;
        db.flush()?;
        println!("Successfully stored {} entries.", count);
        Ok(Self { db })
    }

    pub fn is_empty(&self) -> bool {
        self.db.is_empty()
    }

    pub fn search(
        &self,
        query: String,
        lines: usize,
        offset: usize,
    ) -> Result<Vec<(String, String)>> {
        let query = query.as_bytes();
        let mut results = Vec::new();
        for item in self.db.range(query..).skip(offset).take(lines) {
            let (key, value) = item?;
            let key = String::from_utf8(key.to_vec())?;
            let value = String::from_utf8(value.to_vec())?;
            results.push((key, value))
        }
        Ok(results)
    }
}
