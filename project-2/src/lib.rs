//! # KvStore
//! Simple Key Value Store
#![deny(missing_docs)]
use clap::Error;
use error::CustomError;
use serde::ser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::hash::Hash;
use std::io::BufRead;
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
        let path = path.as_ref().join("storage.json");
        //dbg!(path.to_str());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        //dbg!(&file);
        let mut storage: HashMap<String, String> = HashMap::new();
        for line in std::io::BufReader::new(&file).lines() {
            let line = line?;
            // dbg!(&line);
            let transaction: Transaction = serde_json::from_str(&line)?;
            match transaction {
                Transaction::Set(key, value) => {
                    storage.insert(key, value);
                }
                Transaction::Remove(key) => {
                    storage.remove(&key);
                }
            }
        }
        Ok(KvStore { storage, file })
    }

    /// Set a key to a value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.storage.insert(key.clone(), value.clone());
        let transaction = Transaction::Set(key, value);
        let serialized = serde_json::to_string(&transaction)? + "\n";
        // dbg!(&serialized);
        self.file.write_all(serialized.as_bytes())?;
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
        let serialized = serde_json::to_string(&transaction)? + "\n";
        // dbg!(&serialized);
        self.file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}
