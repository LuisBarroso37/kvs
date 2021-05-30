use std::process;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};

use crate::{KvStore, KvsError, Result};

#[derive(Debug, StructOpt, PartialEq, Serialize, Deserialize)]
/// Arguments for get subcommand
pub struct GetArgs {
    /// Key which will holds the value
    pub key: String,
}

#[derive(Debug, StructOpt, PartialEq, Serialize, Deserialize)]
/// Arguments for set subcommand
pub struct SetArgs {
    /// Key which will holds the value
    pub key: String,
    /// Value associated with given key
    pub value: String,
}

#[derive(Debug, StructOpt, PartialEq, Serialize, Deserialize)]
/// Arguments for rm subcommand
pub struct RmArgs {
    /// Key which will holds the value
    pub key: String,
}

#[derive(Debug, StructOpt, PartialEq, Serialize, Deserialize)]
#[structopt(name = "subcommand", about = "Pngme commands")]
/// Subcommands of command line interface
pub enum Command {
    /// Gets the string value of a given string key
    /// Prints an error and returns a non-zero exit code on failure
    Get(GetArgs),
    /// Sets the value of a string key to a string
    /// Prints an error and returns a non-zero exit code on failure
    Set(SetArgs),
    /// Removes a given key
    /// Prints an error and returns a non-zero exit code on failure
    Rm(RmArgs),
}

/// Run code associated with each subcommand
pub fn run(command: Command, store: &mut KvStore) -> Result<()> {
    match command {
        Command::Get(args) => match store.get(args.key) {
            Ok(Some(value)) => println!("{}", value),
            Ok(None) => println!("{}", KvsError::KeyNotFound),
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1)
            }
        },
        Command::Set(args) => {
            match store.set(args.key, args.value) {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1)
                }
            }
        },
        Command::Rm(args) => match store.remove(args.key) {
            Ok(()) => process::exit(0),
            Err(e) => {
                println!("{}", e);
                process::exit(1)
            }
        },
    }

    Ok(())
}

#[derive(StructOpt)]
/// Struct which represents the program's parsed command line arguments
pub struct Opt {
    #[structopt(subcommand)]
    /// Subcommands of command line interface
    pub command: Command,
}