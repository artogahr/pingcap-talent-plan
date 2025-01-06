//! # KvStore
//! Simple Key Value Store
#![deny(missing_docs)]
use std::collections::HashMap;
use error::CustomError;
use serde_json;
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
}

impl KvStore {
    /// Create a new Key Value Store
    pub fn new() -> KvStore {
        KvStore {
            storage: HashMap::new(),
        }
    }

    /// Open a Key Value Store from a file
    pub fn open<F: AsRef<std::path::Path>>(path: F) -> std::io::Result<KvStore> {
        let path = path.as_ref();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let reader = std::io::BufReader::new(file);
        let storage: HashMap<String, String> = serde_json::from_reader(reader)?;
        Ok(KvStore { storage })
    }

    /// Set a key to a value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.storage.insert(key, value);
        Ok(()) 
    }

    /// Get a value ass. with a key
    pub fn get(&self, key: String) -> Result<Option<String>>{
        match self.storage.get(&key).cloned() {
            Some(val) => Ok(Some(val)),
            None => Ok(None)
        }
    }

    /// Remove a key with it's value from the store
    pub fn remove(&mut self, key: String) -> Result<()>{
        self.storage.remove(&key);
        Ok(())
    }
}
