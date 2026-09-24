//! Integration test for the `ARINSERT` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARINSERT").arg("foo").query::<Value>(&mut con);
    assert_arity_error(result);

    // No "too many args" check, as `ARINSERT` consumes all the items that are there.

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
    // And we're inserting multiple items in one go.
    let res = con.arinsert("foo", &["quux", "quuux", "quuuux"]).unwrap();
    assert_eq!(res, 3);

    assert_eq!(con.arget("foo", 0).unwrap().unwrap(), "bar");
    assert_eq!(con.arget("foo", 1).unwrap().unwrap(), "quux");
    assert_eq!(con.arget("foo", 2).unwrap().unwrap(), "quuux");
    assert_eq!(con.arget("foo", 3).unwrap().unwrap(), "quuuux");
    assert_eq!(con.arget("foo", 42).unwrap().unwrap(), "baz");
}
