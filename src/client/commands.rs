use std::net::SocketAddr;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};

#[derive(Debug, StructOpt, PartialEq, Serialize, Deserialize)]
/// Command types received from the command line interface
pub enum Command {
    /// Get the string value of a given string key
    Get { key: String },
    /// Set the value of a string key to a string
    Set { key: String, value: String},
    /// Remove a given string key
    #[structopt(name="rm")]
    Remove { key: String },
}

#[derive(StructOpt)]
/// Struct which represents the client's parsed command line arguments
pub struct ClientOpt {
    #[structopt(subcommand)]
    /// Subcommands of command line interface
    pub command: Command,
    #[structopt(
        default_value = "127.0.0.1:4000", 
        long="addr",
        value_name = "IP:PORT",
        parse(try_from_str)
    )]
    /// Connection IP address
    pub addr: SocketAddr
}