use std::net::SocketAddr;
use std::str::FromStr;
use std::fmt::{self, Display};
use structopt::StructOpt;

use crate::KvsError;

#[derive(StructOpt)]
/// Struct which represents the server's parsed command line arguments
pub struct ServerOpt {
    #[structopt(
        default_value = "127.0.0.1:4000", 
        long, 
        value_name = "IP:PORT",
        parse(try_from_str)
    )]
    /// Listening IP address
    pub addr: SocketAddr,
    
    #[structopt(
        default_value = "kvs",
        long, 
        value_name = "ENGINE-NAME",
        possible_values = &Engine::variants()
    )]
    /// Storage Engine
    pub engine: Engine
}

#[derive(Debug, StructOpt, PartialEq, Eq)]
pub enum Engine {
    Kvs,
    Sled
}

impl Engine {
    /// Possible values of this enum
    fn variants() -> [&'static str; 2] {
        ["kvs", "sled"]
    }
}

impl FromStr for Engine {
    type Err = KvsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "kvs" => Ok(Engine::Kvs),
            "sled" => Ok(Engine::Sled),
            _ => Err(KvsError::UnknownEngine)
        }
    }
}

impl Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Engine::Kvs => "kvs",
            Engine::Sled => "sled",
        };
        write!(f, "{}", printable)
    }
}