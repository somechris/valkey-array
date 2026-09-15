//! Utilities for unit tests

use valkey_module::ValkeyString;

pub fn vkstr<S: Into<String>>(input: S) -> ValkeyString {
    ValkeyString::test(input.into())
}
