use crate::commands::Command;
use std::collections::HashMap;

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
        
        match execution {
            Ok(_) => println!("Sniffing completed"),
            Err(e) => println!("Error: {:?}", e)
        }
    }

    fn show_usage() {
        todo!()
    }
}

#[cfg(test)]
mod test
{
    use std::collections::HashMap;
    use crate::commands::Command;

    #[tokio::test]
    async fn text_execute_with_mysql()
    {
        let mut flags = HashMap::new();
        flags.insert("-u".to_string(), "mysql://LOCAL_ADMIN:abc123.@test-db.lebastudios.org:3306".to_string());
        
        crate::commands::sniff::Sniff::execute(flags).await;
    }
}