use std::error;
use std::io;
use std::fmt;
use std::result;

/// Holds any kind of error.
pub type Error = KvsError;

/// Holds a `Result` of any kind of error.
pub type Result<T> = result::Result<T, Error>;

/// KvsError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum KvsError {
    /// Represents a failure to find the given key in the store.
    KeyNotFound,

    /// Represents retrieving an unexpected command from the store
    UnexpectedCommand,

    /// Represents a failure to serialize or deserialize data
    SerializationError(serde_json::Error),

    /// Represents all other cases of `std::io::Error`.
    IOError(io::Error),
}

impl error::Error for KvsError {}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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