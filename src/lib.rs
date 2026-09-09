//! `valkey-array` is a [Valkey](https://valkey.io/) module implementing array commands (`ARSET`, `ARGET`, ...)
//!
//! ## Supported commands
//!
//! * [`ARCOUNT`](commands::arcount()) - gets the number of used entries of an array
//! * [`ARDEL`](commands::ardel()) - deletes values from an array
//! * [`ARGET`](commands::arget()) - gets values from an array
//! * [`ARLEN`](commands::arlen()) - gets an array's highest used position + 1
//! * [`ARSET`](commands::arset()) - sets values in an array
//!
//! ## Usage
//!
//! 1. Build the module (see [README.md](https://github.com/somechris/valkey-array/blob/main/README.md)).
//! 2. Start Valkey with the module
//!
//!   ```
//!   valkey-server --loadmodule path/to/valkey-array/target/release/libvalkey_array.so
//!   ```
//! 3. In a different terminal, run `redis-cli` and test with the following commands:
//!
//!   ```shell
//!   127.0.0.1:6379> arset foo 2 "bar"
//!   (integer) 0
//!   127.0.0.1:6379> arset foo 42 "baz"
//!   (integer) 0
//!   127.0.0.1:6379> arget foo 2
//!   "bar"
//!   127.0.0.1:6379> arget foo 42
//!   "baz"
//!   127.0.0.1:6379> arset foo 2 "quux"
//!   (integer) 0
//!   127.0.0.1:6379> arget foo 2
//!   "quux"
//!   127.0.0.1:6379> arget foo 23
//!   (nil)
//!   ```
//!
//! ## Caveats
//!
//! This implementation is a rough proof of concepts.
//!
//! It's especially missing:
//!
//! * many commands
//! * efficient use of memory
//! * replication support
//! * 1:1 compatibility with [Redis' array commands](https://redis.io/docs/latest/develop/data-types/arrays/)

pub mod commands;
pub mod types;

use crate::types::VKARRAY;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::valkey_module;

valkey_module! {
    name: "vkarray",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [VKARRAY],
    commands: [
        ["arcount", commands::arcount, "", 1, 1, 1],
        ["ardel", commands::ardel, "", 1, 1, 1],
        ["arget", commands::arget, "", 1, 1, 1],
        ["arlen", commands::arlen, "", 1, 1, 1],
        ["arset", commands::arset, "", 1, 1, 1],
    ],
}
