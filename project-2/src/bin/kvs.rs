use clap::{Parser, Subcommand};
use kvs::{KvStore, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Set { key, value }) => {
            let mut storage = KvStore::open(".")?;
            storage.set(key.clone(), value.clone())?;
            Ok(())
        }
        Some(Commands::Get { key }) => {
            let storage = KvStore::open(".")?;
            match storage.get(key.clone())? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
            Ok(())
        }
        Some(Commands::Rm { key }) => {
            let mut storage = KvStore::open(".")?;
            match storage.remove(key.clone()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("{}", e);
                    Err(e)
                }
            }
        }
        None => {
            std::process::exit(1);
        }
    }
}
