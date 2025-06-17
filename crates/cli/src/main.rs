mod commands;

use std::env;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum Error
{
    NotEnoughArgsIntroduced
}

#[tokio::main]
async fn main()
{
    let command = env::args().skip(1).collect::<Vec<String>>().join(" ");

    match commands::try_execute(&command).await {
        Ok(_) => (),
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
}
