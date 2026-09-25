//! `valkey-array` is a [Valkey](https://valkey.io/) module implementing array commands (`ARSET`, `ARGET`, ...)
//!
//! ## Supported commands
//!
//! * [`ARCOUNT`](commands::arcount()) - gets the number of used entries of an array
//! * [`ARDEL`](commands::ardel()) - deletes values from an array
//! * [`ARDELRANGE`](commands::ardelrange()) - deletes a range of values from an array
//! * [`ARGET`](commands::arget()) - gets values from an array
//! * [`ARGETRANGE`](commands::argetrange()) - gets a range of values from an array
//! * [`ARGREP`](commands::argrep()) - searches for key/values in a range
//! * [`ARINFO`](commands::arinfo()) - gives information about the array
//! * [`ARINSERT`](commands::arinsert()) - inserts elements into the array
//! * [`ARLASTITEMS`](commands::arlastitems()) - gives values up to the current insert position
//! * [`ARLEN`](commands::arlen()) - gets an array's highest used position + 1
//! * [`ARMGET`](commands::armget()) - gets multiple values from an array
//! * [`ARMSET`](commands::armset()) - sets multiple values in an array
//! * [`ARNEXT`](commands::arnext()) - gets the position for the next insert
//! * [`AROP`](commands::arop()) - runs an operation on a range of an array
//! * [`ARRING`](commands::arring()) - insert an element in ring-buffer fashion
//! * [`ARSCAN`](commands::arscan()) - gets existing positions and values in a range
//! * [`ARSEEK`](commands::arseek()) - sets the position for the next insert
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
//!   127.0.0.1:6379> arset foo 23 "bar"
//!   (integer) 1
//!   127.0.0.1:6379> arset foo 42 "baz"
//!   (integer) 1
//!   127.0.0.1:6379> arget foo 23
//!   "bar"
//!   127.0.0.1:6379> arget foo 42
//!   "baz"
//!   127.0.0.1:6379> arcount foo
//!   (integer) 2
//!   127.0.0.1:6379> arlen foo
//!   (integer) 43
//!   127.0.0.1:6379> ardel foo 42
//!   (integer) 1
//!   127.0.0.1:6379> arget foo 42
//!   (nil)
//!   127.0.0.1:6379> arcount foo
//!   (integer) 1
//!   127.0.0.1:6379> arlen foo
//!   (integer) 24
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

mod array;
pub mod commands;
pub mod registration;

pub mod utils;

pub use array::Array;
