use std::error;
use std::io;
use std::fmt;
use std::result;
use std::string::FromUtf8Error;

/// Holds any kind of error.
pub type Error = KvsError;

/// Holds a `Result` of any kind of error.
pub type Result<T> = result::Result<T, Error>;

/// KvsError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum KvsError {
    /// Represents a failure to find the given key in the store.
    KeyNotFound,

    /// Represents retrieving an unexpected command from the store.
    /// It indicates a corrupted log or a program bug.
    UnexpectedCommand,

    /// Represents a failure to serialize or deserialize data.
    SerializationError(serde_json::Error),

    /// Represents all errors of `std::io::Error`.
    IOError(io::Error),

    /// Represents trying to parse a string into a non-existing database engine type.
    UnknownEngine,

    /// Represents an error received when engine parsed from command line
    /// does not match the engine set in the config file
    InvalidEngine(String),

    /// Represents an error received from the kvs server.
    RequestError(String),

    /// Represents all errors of the Sled engine.
    SledError(sled::Error),

    /// Represents a parsing error when trying to convert a value retrieved from
    /// the sled engine into a UTF-8 sequence
    Utf8Error(FromUtf8Error)
}

impl error::Error for KvsError {}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KvsError::KeyNotFound => {
                write!(f, "Key not found")
            },
            KvsError::SerializationError(ref err) => {
                err.fmt(f)
            },
            KvsError::UnexpectedCommand => {
                write!(f, "Unexpected command")
            },
            KvsError::IOError(ref err) => {
                err.fmt(f)
            },
            KvsError::UnknownEngine => {
                write!(f, "Unknown database engine")
            },
            KvsError::RequestError(e) => {
                write!(f, "Error from server: {}", e)
            },
            KvsError::SledError(ref err) => {
                err.fmt(f)
            },
            KvsError::Utf8Error(ref err) => {
                err.fmt(f)
            },
            KvsError::InvalidEngine(engine) => {
                write!(f, "Invalid choosen engine. Your previously set engine in the config file was {}", engine)
            }
        }
    }
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::IOError(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::SerializationError(err)
    }
}

impl From<sled::Error> for KvsError {
    fn from(err: sled::Error) -> KvsError {
        KvsError::SledError(err)
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(err: FromUtf8Error) -> KvsError {
        KvsError::Utf8Error(err)
    }
}