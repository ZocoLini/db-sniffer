mod help;
mod sniff;
mod version;

use crate::commands::help::Help;
use crate::commands::sniff::Sniff;
use crate::commands::version::Version;
use std::collections::HashMap;

pub trait Command {
    async fn execute(flags: HashMap<String, &str>);
    fn show_usage();
}

pub async fn try_execute(args: &str) -> Result<(), crate::Error> {
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

fn map_flags(args: &Vec<String>) -> HashMap<String, &str> {
    let mut hash_map = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let actual_word = &args[i];
        let next_word = &args[i + 1];

        #[cfg(debug_assertions)]
        {
            println!("Arg found: {actual_word}");
        }

        if is_flag(actual_word) {
            if i + 1 < args.len() && !is_flag(next_word) {
                let flag_value = if next_word.ends_with("\"") && next_word.starts_with("\"") {
                    &next_word[1..next_word.len() - 1]
                } else {
                    next_word
                };

                #[cfg(debug_assertions)]
                {
                    println!("Arg found: {flag_value}");
                }

                hash_map.insert(actual_word.to_string(), flag_value);
                i += 2;
            } else {
                hash_map.insert(actual_word.to_string(), "");
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    hash_map
}

fn is_flag(s: &str) -> bool {
    s.starts_with("-") || s.starts_with("--")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_flags() {
        let mut args = Vec::new();

        vec![
            "-n",
            "name",
            "a",
            "-a",
            "-p",
            "\"path dir/pe_pe\"",
            "-r",
            "-as-dir",
            "--url",
            "abc",
        ]
        .iter()
        .for_each(|s| args.push(s.to_string()));

        let result = map_flags(&args);

        assert_eq!(result.get("-n").unwrap().to_string(), "name");
        assert_eq!(result.get("-p").unwrap().to_string(), "path dir/pe_pe");
        assert_eq!(result.get("-r").unwrap().to_string(), "");
        assert_eq!(result.get("-a").unwrap().to_string(), "");
        assert_eq!(result.get("-as-dir").unwrap().to_string(), "");
        assert_eq!(result.get("--url").unwrap().to_string(), "abc");

        let mut args = Vec::new();

        vec!["mkt", "remove", "-n", "crates"]
            .iter()
            .for_each(|s| args.push(s.to_string()));

        let result = map_flags(&args);

        assert_eq!(result.get("-n").unwrap().to_string(), "crates");
    }

    #[test]
    fn test_is_flag() {
        assert_eq!(is_flag("-n"), true);
        assert_eq!(is_flag("--name"), true);
        assert_eq!(is_flag("name"), false);
    }
}
