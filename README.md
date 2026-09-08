# valkey-array

valkey-array is a [Valkey](https://valkey.io/) module implementing array commands (`ARSET`, `ARGET`, ...)

## Usage

* Install Rust, and `git`
* Run

```
git clone https://github.com/somechris/valkey-array
cd valkey-array
cargo build --release
```

* The compiled module is now under `target/release`

To start Valkey with the module, run

```
valkey-server --loadmodule path/to/valkey-array/target/release/libvalkey_array.so
```

Then in a different terminal, run `redis-cli` and test with the following commands:

```
127.0.0.1:6379> arset foo 2 "bar"
(integer) 0
127.0.0.1:6379> arset foo 42 "baz"
(integer) 0
127.0.0.1:6379> arget foo 2
"bar"
127.0.0.1:6379> arget foo 42
"baz"
127.0.0.1:6379> arset foo 2 "quux"
(integer) 0
127.0.0.1:6379> arget foo 2
"quux"
127.0.0.1:6379> arget foo 23
(nil)
```

## Supported commands

* `ARSET` - sets values in an array
* `ARGET` - gets values from an array

## Caveats

This implementation is a rough proof of concepts.

It's especially missing:

* many commands
* efficient use of memory
* replication support
* 1:1 compatibility with [Redis' array commands](https://redis.io/docs/latest/develop/data-types/arrays/)
