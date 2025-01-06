use clap::{Parser, Subcommand};
use kvs::Result;

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
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some(Commands::Get { key }) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some(Commands::Rm { key }) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        None => {
            std::process::exit(1);
        }
    }
    unimplemented!()
}
