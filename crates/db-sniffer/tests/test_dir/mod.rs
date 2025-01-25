use std::{env, fs};

pub fn get() -> String {
    dotenvy::dotenv().ok();

    let test_dir = if let Ok(r) = env::var("TEST_DIR") {
        r
    } else {
        panic!("TEST_DIR env var not found")
    };

    if fs::exists(&test_dir).unwrap_or(false) {
        fs::remove_dir_all(&test_dir).expect("Should empty the test dir");
    }
    
    test_dir
}