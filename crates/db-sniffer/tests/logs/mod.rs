use std::io::Write;
use std::fs::OpenOptions;
use std::time::Instant;

const LOG_FILE_PATH: &str = "integrations-test.log";

pub enum LogLevel {
    Info,
    Error,
    Warning,
}

pub fn log(msg: &str, log_level: LogLevel) {
    let log_level = match log_level {
        LogLevel::Info => "INFO",
        LogLevel::Error => "ERROR",
        LogLevel::Warning => "WARNING",
    };

    let instant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let log_msg = format!("[{}] - {:?} - {}", log_level, instant, msg);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)
        .unwrap();

    writeln!(file, "{}", log_msg).unwrap();
}