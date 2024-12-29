//! # KvStore
//! Simple Key Value Store
#![deny(missing_docs)]
use std::collections::HashMap;

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

    /// Set a key to a value
    pub fn set(&mut self, key: String, value: String) {
        self.storage.insert(key, value);
    }

    /// Get a value ass. with a key
    pub fn get(&self, key: String) -> Option<String> {
        self.storage.get(&key).cloned()
    }

    /// Remove a key with it's value from the store
    pub fn remove(&mut self, key: String) {
        self.storage.remove(&key);
    }
}
