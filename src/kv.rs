use std::collections::{HashMap, BTreeMap};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::fs::{self, File, OpenOptions, create_dir_all, read_dir};
use std::ffi::OsStr;
use serde_json::Deserializer;

use crate::commands::{Command, SetArgs, RmArgs};
use crate::{KvsError, LogPointer, Result};
use crate::{BufReaderWithPos, BufWriterWithPos};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in log files. Log files have
/// increasing id numbers as names with a `log` extension type.
/// A file reader hash map is kept in order to have one reader for each log file.
/// An in-memory 'BTreeMap' stores the keys and the value locations.
///
/// ```rust
/// # use kvs::{KvStore, Result};
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
/// ```
pub struct KvStore {
    /// Directory for saving log files.
    path: PathBuf,
    /// Map with log files' ids as keys and file readers as values.
    readers: HashMap<u64, BufReaderWithPos<File>>,
    /// File writer of the current log file.
    writer: BufWriterWithPos<File>,
    /// Current log file id.
    current_log_id: u64,
    /// In-memory index map with keys coming as the <KEY> value from the command line argument and 
    /// values which are pointers to the location of the corresponding commands saved in the log files.
    index: BTreeMap<String, LogPointer>,
    /// Number of bytes representing "stale" commands that could be
    /// deleted during compaction.
    uncompacted: u64,
}

impl KvStore {
    /// Opens a `KvStore` at the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// # Errors
    ///
    /// It propagates I/O or deserialization errors during the log load.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // Create directory if it does not exist
        let path = path.into();
        create_dir_all(&path)?;
       
        // Get sorted vector of log file ids inside the directory
        let file_ids = sort_log_files(&path)?;
        
        // Instantiate in-memory index map and file readers hash map
        let mut index = BTreeMap::new();
        let mut readers = HashMap::new();
        let mut uncompacted: u64 = 0; // Number of bytes that can be saved after compaction

        for &id in &file_ids {
            // Path to log file
            let filepath = path.join(format!("{}.log", id));

            // Create reader for log file
            let mut reader = BufReaderWithPos::new(File::open(filepath)?);

            // Load log file and get total amount of bytes that can be deleted
            uncompacted += load_log_file(id, &mut reader, &mut index)?;

            // Add reader to hash map
            readers.insert(id, reader);
        }

        // Get file id of last log file and add 1 to it for the new log file
        let current_log_id: u64 = file_ids.last().unwrap_or(&0) + 1;

        // Create writer for new log file (it also creates a reader and adds it to readers hash map)
        let writer = create_new_log_file(&path, current_log_id, &mut readers)?;
        
        Ok(KvStore {
            path,
            readers,
            writer,
            current_log_id,
            index,
            uncompacted,
        })
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    ///
    /// # Errors
    ///
    /// It returns `KvsError::UnexpectedCommand` if the given command is not a Set command.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(cmd) => {
                // Retrieve reader for log file to which the log pointer refers to 
                let reader = self.readers.get_mut(&cmd.log_file_id).expect("Log reader not found");

                // Set the starting position to start reading the command from the log file
                reader.seek(SeekFrom::Start(cmd.start_position))?;

                // Create a smaller reader that will only read the bytes of the command
                let cmd_reader = reader.take(cmd.len);

                // If retrieved command is a Set command, return the value associated with it
                if let Command::Set(args) = serde_json::from_reader(cmd_reader)? {
                    Ok(Some(args.value))
                } else {
                    Err(KvsError::UnexpectedCommand)
                }
            }, 
            None => Ok(None)
        }
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// It propagates I/O or serialization errors while writing to the log
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set(SetArgs {
            key: key.clone(),
            value
        });
        
        // Get last byte's position in the log file
        let pos = self.writer.pos;
        
        // Serialize the command and append it to the file
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        // Create log pointer for the appended command
        let end_pos = self.writer.pos; // Get new last byte's position in the log file
        let value: LogPointer = (self.current_log_id, pos..end_pos).into();
        
        // Insert log pointer in the in-memory index map
        // If the key already existed, add the bytes of the old value to the uncompacted property
        if let Some(old_cmd) = self.index.insert(key, value) {
            self.uncompacted += old_cmd.len;
        };

        // Perform compaction if uncompacted property is bigger than the defined threshold
        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates I/O or serialization errors while writing to the log.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.index.remove(&key) {
            Some(cmd) => {
                // Add removed command's length to the uncompacted property
                self.uncompacted += cmd.len;
        
                // Get last byte's position in the log file
                let pos = self.writer.pos;
                
                // Remove command to be added to the log file
                let cmd = Command::Rm(RmArgs { key: key.clone() });
                
                // Serialize the command and append it to the file
                serde_json::to_writer(&mut self.writer, &cmd)?;
                self.writer.flush()?;

                // Get new last byte's position in the log file
                let end_pos = self.writer.pos;
                
                // Add appended command's length to the uncompacted property
                self.uncompacted += end_pos - pos;

                // Perform compaction if uncompacted property is bigger than the defined threshold
                if self.uncompacted > COMPACTION_THRESHOLD {
                    self.compact()?;
                }

                Ok(())
            },
            None => Err(KvsError::KeyNotFound)
        }
    }

    /// Compaction is performed by going through the log files, finding all the Set commands
    /// that are still in effect and write them to a new log file.
    /// After the write operation is complete, all previous log files are removed.
    pub fn compact(&mut self) -> Result<()> {
        // Set log file id for compaction file
        let compaction_log_file_id = self.current_log_id + 1;

        // Set log file id for new writable log file after compaction is finished
        // The compaction file will be immutable and users will start writing new logs
        // in a new file
        self.current_log_id += 2;
        self.writer = create_new_log_file(
            &self.path, 
            self.current_log_id, 
            &mut self.readers
        )?;

        // Create writer for compaction file
        let mut compaction_writer = create_new_log_file(
            &self.path, 
            compaction_log_file_id, 
            &mut self.readers
        )?;

        // Keep track of the last written byte's position in the compaction file
        let mut pos: u64 = 0;

        // Go through each value in the in-memory index map which are the latest values stored in the database
        for log_pointer in self.index.values_mut() {
            // Get reader of the log file to which the log pointer refers to
            let reader = self.readers.get_mut(&log_pointer.log_file_id).expect("Log reader not found");

            // Make sure reader starts from the start position of the log pointer
            reader.seek(SeekFrom::Start(log_pointer.start_position))?;

            // Create a more specific reader that will only read the bytes that pertain to the log pointer
            let mut cmd_reader = reader.take(log_pointer.len);

            // Copy log pointer to the compaction file and get number of bytes that were copied
            let copied_bytes = io::copy(&mut cmd_reader, &mut compaction_writer)?;

            // Update log pointer in the in-memory index map to refer to the compaction file
            // instead of the original log file
            *log_pointer = (compaction_log_file_id, pos..pos + copied_bytes).into();

            // Add number of bytes copied to the last byte's position tracker
            pos += copied_bytes;
        }

        // Make sure all write operations are completed
        compaction_writer.flush()?;

        // Get all log file ids which are no longer being used
        let old_logs: Vec<u64> = self.readers
            .keys()
            .filter(|&&log_file_id| log_file_id < compaction_log_file_id)
            .copied()
            .collect();

        // Delete unused log files
        for old_log in old_logs.iter() {
            // Delete log file reader
            self.readers.remove(&old_log);

            // Delete log file from directory
            let filepath = self.path.join(format!("{}.log", old_log));
            fs::remove_file(filepath)?;
        }

        // Set KvStore's uncompacted bytes counter to 0
        self.uncompacted = 0;

        Ok(())
    }
}

/// Get sorted vector of log file ids inside the given directory
fn sort_log_files(path: &PathBuf) -> Result<Vec<u64>> {
    let mut file_ids: Vec<u64> = read_dir(&path)?
        .flat_map(|entry| -> Result<_> { Ok(entry?.path()) }) // Get path for each entry in the directory, ignoring errors by using flat_map
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref())) // Filter entries which are files and have .log extension
        .flat_map(|file| { // flat_map ignores None values, keeping only Some(value)
            file.file_name()
            .and_then(OsStr::to_str)
            .map(|s| s.trim_end_matches(".log")) // Remove .log from file name and keep only log file id number
            .map(|s| s.parse::<u64>())
        })
        .flatten() // Ignore errors coming from Result<u64, ParseIntError> and keep only Ok values
        .collect();

    // sort_unstable is faster than stable sort in some cases
    file_ids.sort_unstable();

    Ok(file_ids)
}

/// Load log file and save log pointers of commands to in-memory index map
///
/// Returns the total number of bytes in the file that can be saved in compaction
fn load_log_file(
    id: u64,
    reader: &mut BufReaderWithPos<File>, 
    index: &mut BTreeMap<String, LogPointer>
) -> Result<u64> {
    // Deserialize commands comming from file reader stream
    let mut pos: u64 = reader.seek(SeekFrom::Start(0))?; // Make sure file starts being read from first byte
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;

    // Run loop until None is received from stream.next()
    while let Some(cmd) = stream.next() {
        let end_pos = stream.byte_offset() as u64; // How many bytes were read from the iteration

        match cmd? {
            Command::Set(args) => {
                // Insert returns None if key-value pair did not exist
                // or returns the previous value if it already existed
                if let Some(old_cmd) = index.insert(args.key, (id, pos..end_pos).into()) {
                    // Add old command's bytes to uncompacted counter
                    uncompacted += old_cmd.len;
                }
            },
            Command::Rm(args) => {
                if let Some(old_cmd) = index.remove(&args.key) {
                    // Add old command's bytes to uncompacted counter
                    uncompacted += old_cmd.len;
                };

                // The "remove" command itself can be deleted in the next compaction
                // so we add its length to the uncompacted counter
                uncompacted += end_pos - pos;
            },
            _ => {}
        }

        // end_pos becomes pos for the next iteration
        pos = end_pos;
    }

    Ok(uncompacted)
}

/// Create a new log file with given log file id and add the reader to the readers map.
///
/// Returns the writer to the log.
fn create_new_log_file(
    path: &PathBuf,
    log_file_id: u64, 
    readers: &mut HashMap<u64, BufReaderWithPos<File>>
) -> Result<BufWriterWithPos<File>> {
    // Filepath for new log file
    let filepath = path.join(format!("{}.log", log_file_id));

    // Create writer for new log file
    let writer = BufWriterWithPos::new(
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filepath)?
    )?;

    // Create reader for new log file and add it to readers hash map
    // Reader is created after the writer because the writer creates the file at the given path
    // if it does not exist
    let reader = BufReaderWithPos::new(File::open(&filepath)?);
    readers.insert(log_file_id, reader);

    Ok(writer)
}