use crate::commands::Command;
use std::collections::HashMap;

pub struct Version;

impl Command for Version
{
    async fn execute(_flags: HashMap<String, &str>)
    {
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
    }

    fn show_usage() {}
}