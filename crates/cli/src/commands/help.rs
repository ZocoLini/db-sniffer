use std::collections::HashMap;
use crate::BIN_NAME;
use crate::commands::Command;

pub struct Help;

impl Command for Help {
    async fn execute(_flags: HashMap<String, String>) {
        let help_message = r#"
Usage:
    mkt [add -p <Path to the template you want to add> [-n <Custom name for the template>] [-as-dir]],
        [list],
        [rm -n <Name of the template you want to remove>],
        [spawn -n <Name of the template you want to spawn> [-o <Define an output name>]],
        [help],
        [version]

Options:
    add         Add a new template from the specified path.
                -p <Path>       Path to the template you want to add.
                -n <Name>       Optional: Custom name for the template.
                -as-dir         Optional: Treat the path as a directory template.

    list        List all available templates.
                -d              Optional: Show the description of the templates.

    rm          Remove a template by name.
                -n <Name>       Name of the template to remove.

    spawn       Spawn a template by name.
                -n <Name>       Name of the template to spawn.
                -o <Output>     Optional: Define a custom output name for the spawned template.

    help        Show this help message.

    version     Show the version of the tool.
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