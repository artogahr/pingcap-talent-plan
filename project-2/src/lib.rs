//! # KvStore
//! Simple Key Value Store
#![deny(missing_docs)]
use core::panic;
use error::CustomError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Seek, SeekFrom};
use std::path::PathBuf;
mod error;
pub use error::Result;

/// The basic implementation of the Key Value Store thingy, which uses a HashMap underneath
/// # Examples
/// ```
/// use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_string(), "value".to_string());
/// ```
pub struct KvStore {
    /// The storage for the key value pairs
    /// The key is a string and the value is a tuple of the file number and the offset in the file
    storage: BTreeMap<String, (u32, u64)>,
    /// The folder that the log files are stored in
    folder_path: PathBuf,
    /// The files that the key value pairs are stored in
    /// The key is the file number and the value is the number of expired keys in the file
    files: BTreeMap<u32, u32>,
}

#[derive(Serialize, Deserialize)]
enum Transaction {
    Set(String, String),
    Remove(String),
}

impl KvStore {
    /// Open a Key Value Store from a file
    /// Opening a Key Value Store will read all the files in the folder and
    /// load all the key value pairs
    /// The KVStore struct holds
    /// 1) A map of keys to file numbers and offsets - storage
    /// 2) a folder path that holds the files - folder_path
    /// 3) A map of file numbers to how many expired keys are in the file - files
    pub fn open<F: AsRef<std::path::Path>>(path: F) -> Result<KvStore> {
        let mut storage: BTreeMap<String, (u32, u64)> = BTreeMap::new();
        let mut files: BTreeMap<u32, u32> = BTreeMap::new();
        let mut file_indexes: BTreeSet<u32> = BTreeSet::new();

        // Collect file indexes
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Parse file index from the file name
            if let Some(file_stem) = path.file_stem() {
                if let Ok(file_index) = file_stem.to_string_lossy().parse::<u32>() {
                    file_indexes.insert(file_index);
                } else {
                    // Log a warning for invalid file names (optional)
                    eprintln!("Warning: Skipping file with invalid index: {:?}", path);
                }
            }
        }

        // Process files in sorted order
        for file_index in file_indexes {
            let file_path = path.as_ref().join(format!("{}.bin", file_index));
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&file_path)?;
            let mut reader = BufReader::new(&file);

            loop {
                let pos = reader.stream_position()?;
                match bincode::deserialize_from::<_, Transaction>(&mut reader) {
                    Ok(transaction) => match transaction {
                        Transaction::Set(key, _) => {
                            // Check if the key already exists in storage
                            if let Some(&(old_file_index, _)) = storage.get(&key) {
                                // Increment expired key count for the old file
                                files
                                    .entry(old_file_index)
                                    .and_modify(|count| *count += 1)
                                    .or_insert(1);
                            }
                            // Update storage with the new file index and offset
                            storage.insert(key, (file_index, pos));
                        }
                        Transaction::Remove(key) => {
                            // Check if the key exists in storage
                            if let Some(&(old_file_index, _)) = storage.get(&key) {
                                // Increment expired key count for the old file
                                files
                                    .entry(old_file_index)
                                    .and_modify(|count| *count += 1)
                                    .or_insert(1);
                                // Remove the key from storage
                                storage.remove(&key);
                            }
                        }
                    },
                    Err(e) => {
                        if e.to_string().contains("EOF")
                            || e.to_string().contains("failed to fill whole buffer")
                        {
                            break; // End of file
                        } else {
                            return Err(e.into()); // Propagate other errors
                        }
                    }
                }
            }
        }

        Ok(KvStore {
            storage,
            folder_path: PathBuf::from(path.as_ref()),
            files,
        })
    }

    /// Set a key to a value.
    /// If the key already exists, the old value is marked as expired.

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // Get the current active file index (the highest-numbered file)
        let active_file_index = self.files.keys().max().copied().unwrap_or(0);

        // Check if the active file has exceeded the size limit
        let active_file_path = self.folder_path.join(format!("{}.bin", active_file_index));
        let file_size = fs::metadata(&active_file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // If the file is too large, create a new file
        let (file_index, file_path) = if file_size >= 1000 {
            let new_file_index = active_file_index + 1;
            let new_file_path = self.folder_path.join(format!("{}.bin", new_file_index));
            (new_file_index, new_file_path)
        } else {
            (active_file_index, active_file_path)
        };

        // Open the file in append mode
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&file_path)?;

        // Get the current file position (offset)
        let pos = file.seek(SeekFrom::End(0))?;

        // Serialize and write the `Set` transaction to the file
        let transaction = Transaction::Set(key.clone(), value);
        bincode::serialize_into(&mut file, &transaction)?;

        // If the key already exists, mark the old entry as expired
        if let Some(&(old_file_index, _)) = self.storage.get(&key) {
            self.files
                .entry(old_file_index)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        //println!("Set a new value for {key} in file {file_index} at offset {pos}");
        // Update the storage map with the new file index and offset
        self.storage.insert(key, (file_index, pos));

        // Ensure the new file is tracked in the `files` map
        self.files.entry(file_index).or_insert(0);
        Ok(())
    }

    /// Get the value associated with a key.
    /// Returns `None` if the key does not exist.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        // Look up the key in the storage map
        if let Some(&(file_index, offset)) = self.storage.get(&key) {
            // Construct the file path for the file containing the key
            let file_path = self.folder_path.join(format!("{}.bin", file_index));

            // Open the file in read mode
            let mut file = OpenOptions::new().read(true).open(&file_path)?;

            // Seek to the offset where the key's value is stored
            file.seek(SeekFrom::Start(offset))?;
            //println!("Get value for {key} from file {file_index} at offset {offset}");
            // Deserialize the transaction at the offset
            match bincode::deserialize_from::<_, Transaction>(&mut file)? {
                Transaction::Set(_, value) => {
                    // Return the value if the transaction is a `Set`
                    Ok(Some(value))
                }
                Transaction::Remove(_) => {
                    // This should never happen if the storage map is consistent
                    panic!("Invalid state: Remove transaction found for a valid key");
                }
            }
        } else {
            // Key not found
            Ok(None)
        }
    }

    /// Remove a key and its associated value from the store.
    /// Returns an error if the key does not exist.
    pub fn remove(&mut self, key: String) -> Result<()> {
        // Check if the key exists
        if let Some(&(file_index, _)) = self.storage.get(&key) {
            // Generate a new file index for this write
            let new_file_index = self.get_next_file_index();
            let file_path = self.folder_path.join(format!("{}.bin", new_file_index));

            // Open the file in append mode
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&file_path)?;

            // Serialize and write the `Remove` transaction to the file
            let transaction = Transaction::Remove(key.clone());
            bincode::serialize_into(&mut file, &transaction)?;

            // Mark the old entry as expired
            self.files
                .entry(file_index)
                .and_modify(|count| *count += 1)
                .or_insert(1);

            // Remove the key from the storage map
            self.storage.remove(&key);

            Ok(())
        } else {
            // Key not found
            Err(CustomError::KeyNotFound)
        }
    }

    /// Get the next available file index.
    /// This is a simple implementation that increments the highest existing index.
    fn get_next_file_index(&self) -> u32 {
        self.files.keys().max().map(|&max| max + 1).unwrap_or(0)
    }
}
