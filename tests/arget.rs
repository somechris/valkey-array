//! Integration test for the `ARGET` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::{assert_contains, assert_none, assert_some_eq_x};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARGET")
        .arg("foo")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARGET")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Wrong type for position
    let err = cmd("ARGET")
        .arg("foo")
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arget("bar", 23).unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Getting from an unused key
    let res = con.arget("foo", 42).unwrap();
    assert_none!(res);

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Getting the added element
    let res = con.arget("foo", 42).unwrap();
    assert_some_eq_x!(res, "bar");

    // Getting another (non-existing) element from the key
    let res = con.arget("foo", 23).unwrap();
    assert_none!(res);
}
