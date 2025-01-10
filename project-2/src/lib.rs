//! # KvStore
//! Simple Key Value Store
#![deny(missing_docs)]
use error::CustomError;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::{collections::HashMap, fs::File, io::Write};
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
    storage: HashMap<String, String>,
    file: File,
}

#[derive(Serialize, Deserialize)]
enum Transaction {
    Set(String, String),
    Remove(String),
}

impl KvStore {
    /// Open a Key Value Store from a file
    pub fn open<F: AsRef<std::path::Path>>(path: F) -> Result<KvStore> {
        let path = path.as_ref().join("storage.bincode");
        //dbg!(path.to_str());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        //dbg!(&file);
        let mut storage: HashMap<String, String> = HashMap::new();
        let mut reader = BufReader::new(&file);

        loop {
            match bincode::deserialize_from::<_, Transaction>(&mut reader) {
                Ok(transaction) => match transaction {
                    Transaction::Set(key, value) => {
                        storage.insert(key, value);
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
        Ok(KvStore { storage, file })
    }

    /// Set a key to a value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.storage.insert(key.clone(), value.clone());
        let transaction = Transaction::Set(key, value);
        let serialized = bincode::serialize(&transaction)?;
        // dbg!(&serialized);
        self.file.write_all(&serialized)?;
        Ok(())
    }

    /// Get a value ass. with a key
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.storage.get(&key) {
            Some(key) => Ok(Some(key.clone())),
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
        // dbg!(&serialized);
        self.file.write_all(&serialized)?;
        Ok(())
    }
}
