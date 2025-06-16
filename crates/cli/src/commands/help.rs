use crate::commands::Command;
use std::collections::HashMap;

pub struct Help;

impl Command for Help {
    async fn execute(_flags: HashMap<String, &str>) {
        let version = env!("CARGO_PKG_VERSION");

        println!(
            r#"
db-sniffer v{version} - A database introspector tool (by Borja Castellano)

- [ Commands ] -

 Command   | Description                                         
==========+======================================================
 sniff     | Introspects a database and outputs the results      
 version   | Displays the current version of the tool            
 help      | Displays this help message 

- [ Options ] -

 Options Short / Long     | Type | Description                                                     | Example
 ==========================+======+================================================================+=======================
 -u, --uri                | Str  | Define the connection string to the database                    | -u mysql://user:pass@ip:port/db
 -m, --mode               | Num  | Indicates the generation mode                                   | -m 1
 -o, --out                | Str  | Defines the output variable of the generation mode (optional)   | -o src/main/java/com/example/entities

- [ Generation modes ] -

  Mode | Name
 ======+======
     0 | DDL (Not implemented)
     1 | Hibernate HBM.XML
     2 | Hibernate with JPA Annotations (Not implemented)

Usage: sniffer [command] [options]... 
"#
        );
    }

    fn show_usage() {
        println!("USAGE: {} help ", std::env::args().next().unwrap_or("sniffer".to_string()));
    }
}
