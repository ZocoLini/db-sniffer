use std::path::PathBuf;

pub mod mysql;
pub mod mssql;

fn containers_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_resources")
}