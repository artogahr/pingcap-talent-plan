# KvStore

A simple key-value store implementation in Rust that provides persistent storage with log-structured file organization.

## Features

- Persistent key-value storage
- Log-structured file organization
- Automatic compaction when files exceed size threshold
- Thread-safe operations

## Usage

```rust
use kvs::KvStore;

// Create or open a store
let mut store = KvStore::open("./data")?;

// Set a value
store.set("key".to_string(), "value".to_string())?;

// Get a value
let value = store.get("key".to_string())?;

// Remove a value
store.remove("key".to_string())?;
```

## Implementation Details

- Uses append-only log files for storage
- Implements automatic compaction to prevent unlimited growth
- Maintains an in-memory index for fast lookups
- Handles file corruption gracefully
