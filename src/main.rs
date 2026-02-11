mod db;
mod tui;

use anyhow::{Context, Result, bail};
use clap::Parser;
use db::Db;
use std::path::{Path, PathBuf};

const CMD_NAME: &str = "eijirs";

#[derive(Parser, Debug)]
#[command(name = CMD_NAME, version = "1.0", about = "English-Japanese Dictionary CLI")]
struct Args {
    /// 検索したい単語やフレーズ (指定がない場合はTUIモードで起動)
    #[arg(value_name = "QUERY")]
    query: Option<String>,

    /// 表示する行数 (CLIモード用)
    #[arg(short = 'n', long, value_name = "LINES", default_value_t = 10)]
    lines: usize,

    /// 辞書データベースを初期化する (辞書ファイルのパスを指定)
    #[arg(long = "init", value_name = "DICT_PATH")]
    init: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let db_path = resolve_db_path()?;

    if let Some(dict_path) = args.init {
        return perform_init(&db_path, &dict_path);
    }

    let db = open_db(&db_path)?;

    match args.query {
        Some(query) => perform_search(&db, &query, args.lines),
        None => perform_tui(db),
    }
}

fn resolve_db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Could not determine data directory. Please check your OS environment.")?
        .join(CMD_NAME);

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .context(format!("Failed to create data directory at {:?}", data_dir))?;
    }

    Ok(data_dir.join("db"))
}

/// データベースを初期化・再構築する
fn perform_init(db_path: &Path, dict_path: &Path) -> Result<()> {
    if db_path.exists() {
        std::fs::remove_dir_all(db_path)
            .context("Failed to remove existing database for re-initialization")?;
    }

    println!("Initializing database at {:?}...", db_path);
    Db::init(db_path, dict_path).context("Failed to initialize database")?;

    println!("Successfully initialized.");
    Ok(())
}

fn open_db(db_path: &Path) -> Result<Db> {
    let db = Db::open(&db_path).context("Failed to open database")?;

    if db.is_empty() {
        bail!("Database is empty. Please run with --init <DICT_PATH>");
    }

    Ok(db)
}

fn perform_search(db: &Db, query: &str, lines: usize) -> Result<()> {
    let results = db.search(query.to_string(), lines, 0)?;

    for (key, value) in results {
        println!("{} : {}", key, value);
    }

    Ok(())
}

fn perform_tui(db: Db) -> Result<()> {
    tui::tui_main(db).context("TUI application error")
}
