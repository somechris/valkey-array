//! Utils for integration tests

use redis::{Connection, ConnectionLike, RedisResult, cmd};
use redis_test::utils::CommandMultiArgs;
use redis_test::{TestContext, TestContextBuilder};
use std::collections::HashMap;

const SERVER_BIN_ENV_VAR: &str = "REDISRS_SERVER_BIN";

/// If unset, sets the default server to Valkey
pub fn set_default_server_to_valkey() {
    let mut server_bin = std::env::var_os(SERVER_BIN_ENV_VAR).unwrap_or_default();
    if server_bin.is_empty() {
        // Either the env var was unset, or empty. So we default to Valkey
        server_bin = "valkey-server".into();
    }

    unsafe {
        std::env::set_var(SERVER_BIN_ENV_VAR, server_bin);
    }
}

/// [`TestContextBuilder`] tooling for our integration tests
pub trait ValkeyArrayTestContextBuilder {
    /// Builds a default [`TestContext`] configured for integration tests
    fn build_for_valkey_array() -> TestContext;
}

impl ValkeyArrayTestContextBuilder for TestContextBuilder {
    fn build_for_valkey_array() -> TestContext {
        set_default_server_to_valkey();
        TestContextBuilder::new().refine_and_build(|cmd| {
            cmd.arg2("--loadmodule", "target/debug/libvalkey_array.so");
        })
    }
}

/// Typed array commands for [`ConnectionLike`]s
pub trait TypedArrayCommands: ConnectionLike + Sized {
    /// Number of elements in the array
    fn arcount(&mut self, key: &str) -> RedisResult<u64> {
        cmd("ARCOUNT").arg(key).query(self)
    }

    /// Deletes on element from the array
    fn ardel(&mut self, key: &str, position: u64) -> RedisResult<u64> {
        cmd("ARDEL").arg(key).arg(position).query(self)
    }

    /// Deletes a range of elements
    fn ardelrange(&mut self, key: &str, start: u64, end: u64) -> RedisResult<u64> {
        cmd("ARDELRANGE").arg(key).arg(start).arg(end).query(self)
    }

    /// Gets an element from the array
    fn arget(&mut self, key: &str, position: u64) -> RedisResult<Option<String>> {
        cmd("ARGET").arg(key).arg(position).query(self)
    }

    /// Gets a range of elements
    fn argetrange(&mut self, key: &str, start: u64, end: u64) -> RedisResult<Vec<Option<String>>> {
        cmd("ARGETRANGE").arg(key).arg(start).arg(end).query(self)
    }

    /// Inserts an element into the array at the insert cursor
    fn arinfo(&mut self, key: &str) -> RedisResult<HashMap<String, String>> {
        cmd("ARINFO").arg(key).query(self)
    }

    /// Inserts an element into the array at the insert cursor
    fn arinsert(&mut self, key: &str, value: &str) -> RedisResult<u64> {
        cmd("ARINSERT").arg(key).arg(value).query(self)
    }

    /// Number of highest allocated position + 1
    fn arlen(&mut self, key: &str) -> RedisResult<u64> {
        cmd("ARLEN").arg(key).query(self)
    }

    /// The position for the next insert
    fn arnext(&mut self, key: &str) -> RedisResult<u64> {
        cmd("ARNEXT").arg(key).query(self)
    }

    /// Inserts an element in a ring-buffer fashion
    fn arring(&mut self, key: &str, buffer_size: u64, value: &str) -> RedisResult<u64> {
        cmd("ARRING")
            .arg(key)
            .arg(buffer_size)
            .arg(value)
            .query(self)
    }

    /// Sets the position for the next insert
    fn arseek(&mut self, key: &str, position: u64) -> RedisResult<u64> {
        cmd("ARSEEK").arg(key).arg(position).query(self)
    }

    /// Sets an element in the array
    fn arset(&mut self, key: &str, position: u64, value: &str) -> RedisResult<u64> {
        cmd("ARSET").arg(key).arg(position).arg(value).query(self)
    }
}

impl TypedArrayCommands for Connection {}
