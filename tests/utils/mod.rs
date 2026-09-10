//! Utils for integration tests

use redis_test::utils::CommandMultiArgs;
use redis_test::{TestContext, TestContextBuilder};

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
