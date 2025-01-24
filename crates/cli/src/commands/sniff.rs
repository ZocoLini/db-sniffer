use crate::commands::Command;
use crate::BIN_NAME;
use db_sniffer::generators::hibernate;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

pub enum SniffMode {
    DDL,
    HibernateXML,
    HibernateJPA,
}

impl FromStr for SniffMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(SniffMode::DDL),
            "1" => Ok(SniffMode::HibernateXML),
            "2" => Ok(SniffMode::HibernateJPA),
            _ => Err(()),
        }
    }
}

pub struct Sniff;

impl Command for Sniff {
    async fn execute(flags: HashMap<String, &str>) {
        let uri = match flags.get("-u").or_else(|| flags.get("--url")) {
            Some(uri) => uri,
            None => {
                println!("Missing uri (-u)");
                Self::show_usage();
                return;
            }
        };

        let results = match db_sniffer::sniff(uri).await {
            Ok(a) => a,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };

        let output = match flags.get("-o").or_else(|| flags.get("--out")) {
            Some(&output) => {
                if "." == output {
                    env::current_dir().expect("Failed to get current directory")
                } else {
                    PathBuf::from(output)
                }
            }
            None => env::current_dir().expect("Failed to get current directory"),
        };

        let mode = flags
            .get("-m")
            .or_else(|| flags.get("--mode"))
            .and_then(|m| SniffMode::from_str(m).ok());

        let mode = if let Some(r) = mode {
            r
        } else {
            println!("Missing mode (-m)");
            Self::show_usage();
            return;
        };

        let generator = match mode {
            SniffMode::HibernateXML => hibernate::XMLGenerator::new(&results, &output).unwrap(),
            SniffMode::DDL => panic!("Not implemented mode"),
            SniffMode::HibernateJPA => panic!("Not implemented mode"),
        };

        generator.generate();
    }

    fn show_usage() {
        println!("USAGE: {} sniff -u <uri> -m <mode> [-o <output>]", BIN_NAME);
    }
}
