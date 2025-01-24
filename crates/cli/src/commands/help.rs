use std::collections::HashMap;
use crate::BIN_NAME;
use crate::commands::Command;

pub struct Help;

// TODO: Better display

impl Command for Help {
    async fn execute(_flags: HashMap<String, String>) {
        let help_message = r#"
sniffer sniff -u <db://user:password@host:port/dbmane> -o <output-dir>
"#;

        println!("{}", help_message);
    }

    fn show_usage() {
        println!(
            "USAGE: {} help ",
            BIN_NAME
        );
    }
}