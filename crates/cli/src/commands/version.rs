use crate::commands::Command;
use std::collections::HashMap;

pub struct Version;

impl Command for Version {
    async fn execute(_flags: HashMap<String, &str>) {
        println!(
            r#"- [Version] -
cli        {}  —  Command line interface for the db-sniffer crate
db-sniffer {}  —  Database introspector"#,
            crate::VERSION,
            db_sniffer::VERSION
        );
    }

    fn show_usage() {}
}
