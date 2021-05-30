//! A simple key/value store.

pub use commands::{Opt, run};
pub use kv::KvStore;
pub use errors::{KvsError, Result};
pub use reader::BufReaderWithPos;
pub use writer::BufWriterWithPos;
pub use log_pointer::LogPointer;

mod kv;
mod commands;
mod errors;
mod reader;
mod writer;
mod log_pointer;