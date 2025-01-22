use crate::commands::Command;
use crate::BIN_NAME;
use db_sniffer::generators::hibernate;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env};

pub struct Sniff;

impl Command for Sniff {
    async fn execute(flags: HashMap<String, String>) {
        let uri = match flags.get("-u") {
            Some(uri) => uri,
            None => {
                Self::show_usage();
                return;
            }
        };

        let execution = db_sniffer::sniff(uri).await;

        let results = match execution {
            Ok(a) => a,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };

        let output = match flags.get("-o") {
            Some(output) => PathBuf::from(output),
            None => env::current_dir().expect("Failed to get current directory"),
        };

        let generator = hibernate::XMLGenerator::new(
            &results,
            &output
        ).unwrap();
        generator.generate();
    }

    fn show_usage() {
        println!("USAGE: {} sniff -u <uri> [-o <output-dir>]", BIN_NAME);
    }
}
