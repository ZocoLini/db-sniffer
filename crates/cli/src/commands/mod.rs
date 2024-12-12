mod help;
mod sniff;
mod version;

use std::collections::HashMap;
use crate::commands::help::Help;
use crate::commands::sniff::Sniff;
use crate::commands::version::Version;

pub trait Command
{
    async fn execute(flags: HashMap<String, String>);
    fn show_usage();
}

pub async fn try_execute(args: &str) -> Result<(), crate::Error>
{
    let mut parts = args.split_whitespace();

    let main_command = match parts.next() {
        Some(command) => command,
        None => return Err(crate::Error::NotEnoughArgsIntroduced),
    };

    let command_instr = parts.map(String::from).collect::<Vec<String>>();
    let flags = map_flags(&command_instr);

    match main_command {
        "sniff" => Ok(Sniff::execute(flags).await),
        "version" => Ok(Version::execute(flags).await),
        _ => Ok(Help::execute(flags).await),
    }
}

fn map_flags(args: &Vec<String>) -> HashMap<String, String>
{
    let mut hash_map = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let actual_word = &args[i];

        if is_flag(actual_word) {
            if i + 1 < args.len() && !is_flag(&args[i + 1]) {
                hash_map.insert(actual_word.to_string(), args[i + 1].clone());
                i += 2;
            } else {
                hash_map.insert(actual_word.to_string(), String::new());
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    hash_map
}

fn is_flag(s: &str) -> bool
{
    s.starts_with("-") || s.starts_with("--")
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_map_flags()
    {
        let args = vec!["-n", "name", "a", "-a", "-p", "path", "-r", "-as-dir"];
        let result = map_flags(&args.iter().map(|s| s.to_string()).collect());

        assert_eq!(result.get("-n").unwrap(), "name");
        assert_eq!(result.get("-p").unwrap(), "path");
        assert_eq!(result.get("-r").unwrap(), "");
        assert_eq!(result.get("-a").unwrap(), "");
        assert_eq!(result.get("-as-dir").unwrap(), "");

        let args = vec!["mkt", "remove", "-n", "crates"];
        let result = map_flags(&args.iter().map(|s| s.to_string()).collect());

        assert_eq!(result.get("-n").unwrap(), "crates");
    }

    #[test]
    fn test_is_flag()
    {
        assert_eq!(is_flag("-n"), true);
        assert_eq!(is_flag("--name"), true);
        assert_eq!(is_flag("name"), false);
    }
}