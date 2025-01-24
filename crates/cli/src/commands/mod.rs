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
    let mut parts = split_args(args).into_iter();

    let main_command = match parts.next() {
        Some(command) => command,
        None => return Err(crate::Error::NotEnoughArgsIntroduced),
    };

    let command_instr = parts.map(String::from).collect::<Vec<String>>();
    let flags = map_flags(&command_instr);

    match main_command.as_str() {
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
                let flag_value = if next_word.ends_with("\"")
                    && next_word.starts_with("\"")
                    && next_word.len() > 1
                {
                    if next_word.len() == 2 {
                        ""
                    } else {
                        &next_word[1..next_word.len() - 1]
                    }
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

fn split_args(args: &str) -> Vec<String> {
    let mut start_index = 0;
    
    let mut result = Vec::new();
    
    let mut inside_quotes = false;

    for (i, c) in args.chars().enumerate() {
        if c == ' ' && !inside_quotes {
            if start_index != i {
                insert_range(args, start_index, i, &mut result);
            }
            
            start_index = i + 1;
        } else if c == '"' && args.chars().nth(i - 1).unwrap() != '\\' {
            inside_quotes = !inside_quotes;
        }
    }

    if start_index != args.len() {
        insert_range(args, start_index, args.len(), &mut result);
    }
    
    return result;
    
    fn insert_range(args: &str, start: usize, end: usize, result: &mut Vec<String>) {
        let text = &args[start..end];
        
        let t = if text.starts_with("\"") && text.ends_with("\"") {
            &text[1..text.len() - 1]
        } else {
            text
        };
        
        let t = t.replace("\\\"", "\"");
        
        result.push(t);
    }
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
    
    #[test]
    fn test_split_args() {
        assert_eq!(split_args("mkt remove -n crates"), vec!["mkt", "remove", "-n", "crates"]);
        assert_eq!(split_args("mkt remove -n crates      "), vec!["mkt", "remove", "-n", "crates"]);
        assert_eq!(split_args("mkt \"remove -n\" crates -a"), vec!["mkt", "remove -n", "crates", "-a"]);
        assert_eq!(split_args("mkt remove -n crates \"-a       \""), vec!["mkt", "remove",  "-n", "crates", "-a       "]);
        assert_eq!(split_args("mkt remove -n crates \"-a       "), vec!["mkt", "remove", "-n", "crates", "\"-a       "]);
        
        assert_eq!(split_args(""), Vec::<&str>::new());
        
        assert_eq!(split_args(r#"mkt \"remove -n crates"#), vec!["mkt", "\"remove", "-n", "crates"]);
        assert_eq!(split_args("mkt remove -n crates \\\"-a       "), vec!["mkt", "remove", "-n", "crates", "\"-a"]);
    }
}
