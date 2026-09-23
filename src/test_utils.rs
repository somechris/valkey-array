//! Utilities for unit tests

#![allow(clippy::expect_used, reason = "This is only used in test code")]
#![allow(clippy::panic, reason = "This is only used in test code")]

use std::fmt::Debug;
use valkey_module::{ValkeyString, ValkeyValue};

/// Builds a new [`ValkeyString`]
pub fn vkstr<S: Into<String>>(input: S) -> ValkeyString {
    ValkeyString::test(input.into())
}

/// Converts [`u32`]s to a list of [`ValkeyValues`]s
pub fn u32s_to_vec_value(items: &[i32]) -> Vec<ValkeyValue> {
    items.iter().map(|x| (*x as i64).into()).collect()
}

/// Asserts that the given [`Result`] is a position error
pub fn assert_position_error<OK: Debug, ERR: ToString>(res: Result<OK, ERR>) {
    let err = res.expect_err("position should fail");
    let err_msg = err.to_string();
    if !err_msg.contains("invalid") || !err_msg.contains("position") {
        panic!("expected position error, got {err_msg}");
    }
}
