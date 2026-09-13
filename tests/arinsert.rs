//! Integration test for the `ARINSERT` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::assert_contains;
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARINSERT")
        .arg("foo")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARINSERT")
        .arg("foo")
        .arg("bar")
        .arg("baz")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arinsert("bar", "quux").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Inserting the first element to the array
    let res = con.arinsert("foo", "bar").unwrap();
    assert_eq!(res, 0);
    assert_eq!(con.arget("foo", 0).unwrap().unwrap(), "bar");

    // Setting an element at a higher position
    con.arset("foo", 42, "baz").unwrap();

    // Inserting should continue at position 1 (not 43 -- next available after last set)
    let res = con.arinsert("foo", "quux").unwrap();
    assert_eq!(res, 1);

    assert_eq!(con.arget("foo", 0).unwrap().unwrap(), "bar");
    assert_eq!(con.arget("foo", 1).unwrap().unwrap(), "quux");
    assert_eq!(con.arget("foo", 42).unwrap().unwrap(), "baz");
}
