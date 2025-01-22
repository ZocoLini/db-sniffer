use std::path::PathBuf;

pub(crate) mod mysql;

fn containers_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_resources")
}