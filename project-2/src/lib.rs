//! # KvStore
//! Simple Key Value Store
#![deny(missing_docs)]
use core::panic;
use error::CustomError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
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
    storage: BTreeMap<String, u64>,
    file_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
enum Transaction {
    Set(String, String),
    Remove(String),
}

impl KvStore {
    /// Open a Key Value Store from a file
    pub fn open<F: AsRef<std::path::Path>>(path: F) -> Result<KvStore> {
        let file_path: PathBuf = path.as_ref().join("storage.bincode");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;
        let mut storage: BTreeMap<String, u64> = BTreeMap::new();
        let mut reader = BufReader::new(&file);

        loop {
            let pos = reader.stream_position()?;
            match bincode::deserialize_from::<_, Transaction>(&mut reader) {
                Ok(transaction) => match transaction {
                    Transaction::Set(key, _) => {
                        storage.insert(key, pos);
                    }
                    Transaction::Remove(key) => {
                        storage.remove(&key);
                    }
                },
                Err(e) => {
                    if e.to_string().contains("EOF")
                        || e.to_string().contains("failed to fill whole buffer")
                    {
                        break;
                    }
                    return Err(e.into());
                }
            }
        }
        Ok(KvStore { storage, file_path })
    }

    /// Set a key to a value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .open(self.file_path.clone())?;
        let pos = file.metadata()?.len();
        self.storage.insert(key.clone(), pos);
        let transaction = Transaction::Set(key, value);
        let serialized = bincode::serialize(&transaction)?;
        file.write(&serialized)?;
        Ok(())
    }

    /// Get a value ass. with a key
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.storage.get(&key) {
            Some(&offset) => {
                let mut file = OpenOptions::new().read(true).open(self.file_path.clone())?;
                let _ = file.seek(SeekFrom::Start(offset))?;
                match bincode::deserialize_from::<_, Transaction>(&mut file) {
                    Ok(transaction) => match transaction {
                        Transaction::Set(_, value) => return Ok(Some(value)),
                        Transaction::Remove(..) => {
                            panic!("A remove command read for a set command offset")
                        }
                    },
                    Err(e) => {
                        if e.to_string().contains("EOF")
                            || e.to_string().contains("failed to fill whole buffer")
                        {
                        }
                        return Err(e.into());
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Remove a key with it's value from the store
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.storage.contains_key(&key) {
            return Err(CustomError::KeyNotFound);
        }
        self.storage.remove(&key);
        let transaction = Transaction::Remove(key);
        let serialized = bincode::serialize(&transaction)?;
        let mut file = OpenOptions::new()
            .append(true)
            .open(self.file_path.clone())?;
        file.write(&serialized)?;
        Ok(())
    }
}
