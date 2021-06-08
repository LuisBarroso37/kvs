use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use serde_json::Deserializer;
use slog::{info, error, debug};

use crate::{Command, KvsEngine , CommandResponse, Result};

pub struct KvsServer {
  addr: SocketAddr,
  engine: Box<dyn KvsEngine>,
  logger: slog::Logger
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: Box<dyn KvsEngine>, logger: slog::Logger) -> Self {
        Self { addr, engine, logger }
    }

    /// Run server
    pub fn run(&mut self) -> Result<()> {
        info!(self.logger, "Listening on {}", &self.addr);
        info!(self.logger, "Version {}", env!("CARGO_PKG_VERSION"));

        // Bind listener to the address
        let listener = TcpListener::bind(&self.addr)?;

        // Get stream from incoming connections
        for connection in listener.incoming() {
            match connection {
                Ok(stream) => {
                    info!(self.logger, "Connection received: {:?}", &stream);

                    // Create reader for stream
                    let reader = BufReader::new(&stream);

                    // Create deserializer for commands sent through the stream
                    let commands = Deserializer::from_reader(reader).into_iter::<Command>();

                    // Loop through the received commmands until we get None
                    for cmd in commands {
                        debug!(self.logger, "Received command: {:?}", &cmd);

                        // Read command and send response
                        if let Err(e) = self.serve(&stream, cmd?) {
                            error!(self.logger, "Error processing command: {}", e)
                        }
                    }
                },
                Err(e) => error!(self.logger, "Failed to establish a connection: {}", e)
            }
        }

        Ok(())
    }

    /// Check which command was received and send back appropriate response
    pub fn serve (&mut self, stream: &TcpStream, command: Command) -> Result<()> {
        // Create writer for stream
        let mut writer = BufWriter::new(stream);

        // Macro to send back response
        macro_rules! send_res {
            ($res: expr) => {
                let res = $res;
                debug!(self.logger, "Command response: {:?}", &res);

                // Send response back to the stream
                serde_json::to_writer(&mut writer, &res)?;
                writer.flush()?;
            };
        }

        match command {
            Command::Get { key, .. } => match self.engine.get(key) {
                Ok(Some(value)) => {
                    // Set response
                    let res = CommandResponse::Value(value);

                    // Send response back to the stream
                    send_res!(&res);
                },
                Ok(None) => {
                    // Set response
                    let res = CommandResponse::KeyNotFound;

                    // Send response back to the stream
                    send_res!(&res);
                },
                Err(e) => {
                    // Set response
                    let res = CommandResponse::Error(format!("Get command error: {}", e));

                    // Send response back to the stream
                    send_res!(&res);
                }
            },
            Command::Set { key, value, .. } => {
                match self.engine.set(key, value) {
                    Ok(()) => {
                        // Set response
                        let res = CommandResponse::Success;

                        // Send response back to the stream
                        send_res!(&res);
                    },
                    Err(e) => {
                         let res = CommandResponse::Error(format!("Set command error: {}", e));

                        // Send response back to the stream
                        send_res!(&res);
                    }
                }
            },
            Command::Remove { key, .. } => match self.engine.remove(key) {
                Ok(()) => {
                    // Set response
                    let res = CommandResponse::Success;

                    // Send response back to the stream
                    send_res!(&res);
                },
                Err(e) => {
                    // Set response
                    let res = CommandResponse::Error(format!("Remove command error: {}", e));

                    // Send response back to the stream
                    send_res!(&res);
                }
            },
        }

        Ok(())
    }
}