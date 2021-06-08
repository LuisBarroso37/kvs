use std::path::PathBuf;

use crate::{KvsEngine, KvsError, Result};

#[derive(Debug)]
/// Using the "sled" crate, we create a new database engine
pub struct SledKvsEngine {
    db: sled::Db
}


impl SledKvsEngine {
    /// Opens a sled store at the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// # Errors
    ///
    /// It propagates sled errors during the log load.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let db = sled::open(path.into())?;
    
        Ok(Self { db })
    }
}

impl KvsEngine for SledKvsEngine {
    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    ///
    /// # Errors
    ///
    /// It propagates sled errors while reading from the log.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let value = self.db
            .get(key.as_bytes())?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?; // transpose turns an Option<Result<>> into a Result<Option<>>
        
        Ok(value)
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// It propagates sled errors while writing to the log.
    fn set(&mut self, key: String, value: String) -> Result<()> {
        // Set key-value pair in database
        self.db.insert(key, value.as_bytes())?;

        // Make sure the write operation is completed or throws an error
        self.db.flush()?;

        Ok(())
    }

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates sled errors while writing to the log.
    fn remove(&mut self, key: String) -> Result<()> {
        // Remove key-value pair from database
        self.db.remove(key)?.ok_or(KvsError::KeyNotFound)?;

        // Make sure the write operation is completed or throws an error
        self.db.flush()?;

        Ok(())
    }
}