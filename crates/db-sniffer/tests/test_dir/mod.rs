use std::{env, fs};
use std::sync::LazyLock;

static TIMESTAMP: LazyLock<i64> = LazyLock::new(|| {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
});

pub fn get() -> String {
    let test_dir = format!("/tmp/db-sniffer-test-{}", *TIMESTAMP);

    if fs::exists(&test_dir).unwrap_or(false) {
        fs::remove_dir_all(&test_dir).expect("Should empty the test dir");
    }
    
    test_dir
}