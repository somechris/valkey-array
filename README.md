# valkey-array

valkey-array is a [Valkey](https://valkey.io/) module implementing array commands (`ARSET`, `ARGET`, ...)

## Usage

* Install Rust, `git`, and `make`
* Run

```
git clone https://github.com/somechris/valkey-array
cd valkey-array
make install-rust-tooling
make build
```

* The compiled module is now under `target/release`

To start Valkey with the module, run

```
valkey-server --loadmodule path/to/valkey-array/target/release/libvalkey_array.so
```

Then in a different terminal, run `redis-cli` and test with the following commands:

```
127.0.0.1:6379> arset foo 23 "bar"
(integer) 1
127.0.0.1:6379> arset foo 42 "baz"
(integer) 1
127.0.0.1:6379> arget foo 23
"bar"
127.0.0.1:6379> arget foo 42
"baz"
127.0.0.1:6379> arcount foo
(integer) 2
127.0.0.1:6379> arlen foo
(integer) 43
127.0.0.1:6379> ardel foo 42
(integer) 1
127.0.0.1:6379> arget foo 42
(nil)
127.0.0.1:6379> arcount foo
(integer) 1
127.0.0.1:6379> arlen foo
(integer) 24
```

## Supported commands

* `ARCOUNT` - gets the number of used entries of an array
* `ARDEL` - deletes values from an array
* `ARGET` - gets values from an array
* `ARINSERT` - inserts an element into the array
* `ARLEN` - gets an array's highest used position + 1
* `ARNEXT` - gets the position for the next insert
* `ARSET` - sets values in an array

## Caveats

This implementation is a rough proof of concepts.

It's especially missing:

* many commands
* efficient use of memory
* replication support
* 1:1 compatibility with [Redis' array commands](https://redis.io/docs/latest/develop/data-types/arrays/)
