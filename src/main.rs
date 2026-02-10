use anyhow::{Context, Result, bail};
use encoding_rs_io::DecodeReaderBytesBuilder;
use sled::Batch;
use std::fs::File;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

const CMD_NAME: &str = "eijirs";
const BATCH_SIZE: usize = 10_000;

fn get_db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Could not determine data directory")?
        .join(CMD_NAME);

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir.join("db"))
}

fn init_db(db_path: &PathBuf, dict_path: &PathBuf) -> Result<sled::Db> {
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
    Ok(db)
}

#[derive(Parser, Debug)]
#[command(name = CMD_NAME, version = "1.0")]
struct Args {
    #[arg(value_name = "QUERY")]
    query: String,

    #[arg(short = 'n', long, value_name = "LINES", default_value_t = 10)]
    lines: usize,

    #[arg(long = "init", value_name = "DICT_PATH")]
    init: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let db_path = get_db_path()?;
    let db = if let Some(dict_path) = args.init {
        if db_path.exists() {
            std::fs::remove_dir_all(&db_path)?;
        }
        println!("Initializing database at {:?}...", db_path);
        init_db(&db_path, &dict_path)?
    } else {
        sled::open(db_path)?
    };
    if db.is_empty() {
        bail!("Database is empty. Please run with --init <DICT_PATH>");
    }

    let query = args.query.as_bytes();
    for item in db.range(query..).take(args.lines) {
        let (key, value) = item?;
        let key = String::from_utf8(key.to_vec())?;
        let value = String::from_utf8(value.to_vec())?;
        println!("{key} : {value}");
    }

    Ok(())
}
