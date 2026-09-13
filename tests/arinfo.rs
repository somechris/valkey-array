//! Integration test for the `ARINFO` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::{assert_contains, assert_is_empty};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;
use std::collections::HashMap;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARINFO").query::<Value>(&mut con).unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARINFO")
        .arg("foo")
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arinfo("bar").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.arinfo("foo").unwrap();
    assert_is_empty!(res);

    // Set some data in the array
    con.arset("foo", 42, "bar").unwrap();
    con.arset("foo", 23, "baz").unwrap();
    con.arseek("foo", 4711).unwrap();

    // Now there shoud be some proper info
    let res = con.arinfo("foo").unwrap();
    let expected = HashMap::from([
        ("count".to_string(), "2".to_string()),
        ("len".to_string(), "43".to_string()),
        ("insert-cursor".to_string(), "4711".to_string()),
    ]);
    assert_eq!(res, expected);
}
