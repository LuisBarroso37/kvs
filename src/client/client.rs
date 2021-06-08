use serde::Deserialize;
use serde_json::Deserializer;
use slog::{Logger, info, error, debug, warn};
use std::net::{SocketAddr, TcpStream};
use std::io::{BufReader, BufWriter, Write};

use crate::{Command, CommandResponse, KvsError, Result};

pub struct KvsClient {
    addr: SocketAddr,
    logger: Logger
}

impl KvsClient {
     pub fn new(addr: SocketAddr, logger: Logger) -> Self {
        Self { addr, logger }
    }

    /// Run client
    pub fn run(&self, command: Command) -> Result<()> {
        // Connect to kvs-server
        match TcpStream::connect(&self.addr) {
            Ok(stream) => {
                info!(self.logger, "Successfully connected to server in {}", self.addr);
                debug!(self.logger, "Sending command: {:?}", command);

                // Create writer for stream to send command to server
                let mut writer = BufWriter::new(&stream);
                serde_json::to_writer(&mut writer, &command)?;
                writer.flush()?;

                // Create reader for stream to receive response from server
                let reader = BufReader::new(&stream);
                let mut deserializer = Deserializer::from_reader(reader);

                let response = CommandResponse::deserialize(&mut deserializer)?;
                debug!(self.logger, "Received response: {:?}", &response);

                match response {
                    CommandResponse::Value(value) =>  {
                        println!("{}", value);
                        Ok(())
                    },
                    CommandResponse::Success => Ok(()),
                    CommandResponse::KeyNotFound => {
                        warn!(self.logger, "Key not found");
                        println!("Key not found");
                        Ok(())
                    },
                    CommandResponse::Error(e) => {
                        error!(self.logger, "{}", e);
                        Err(KvsError::RequestError(e))
                    }
                }
            },
            Err(e) => {
                error!(self.logger, "Failed to connect: {}", e);
                Err(KvsError::IOError(e))
            }
        }
    }
}