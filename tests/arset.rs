//! Integration test for the `ARSET` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::assert_contains;
use redis::{TypedCommands, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARSET")
        .arg("foo")
        .arg(42)
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARSET")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .arg("baz")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Wrong type for position
    let err = cmd("ARSET")
        .arg("foo")
        .arg("bar")
        .arg("baz")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arset("bar", 23, "quux").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Setting an element a fresh key
    let res = con.arset("foo", 42, "bar").unwrap();
    assert_eq!(res, 1); // 1 as an element got added
    assert_eq!(con.arget("foo", 42).unwrap().unwrap(), "bar");

    // Setting the same element again
    let res = con.arset("foo", 42, "baz").unwrap();
    assert_eq!(res, 0); // 0 as no new slot got added
    assert_eq!(con.arget("foo", 42).unwrap().unwrap(), "baz");
}
