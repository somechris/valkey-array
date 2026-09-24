//! Integration test for the `ARSET` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_position_error,
};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARSET").arg("foo").query::<Value>(&mut con);
    assert_arity_error(result);

    // No "too many args" check, as `ARSET` consumes all the items that are there.

    // Wrong type for position
    let result = cmd("ARSET")
        .arg("foo")
        .arg("bar")
        .arg("baz")
        .query::<Value>(&mut con);
    assert_position_error(result);

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

#[test]
fn multiple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a value that will get overwritten by the below `arset`
    con.arset("foo", 24, "value-24").unwrap();

    let res = con.arset("foo", 23, &["bar", "baz", "quux"]).unwrap();
    assert_eq!(res, 2); // Two new slots got taken (24 was occupied before)

    // Check array contents
    assert_eq!(con.arget("foo", 23).unwrap().unwrap(), "bar");
    assert_eq!(con.arget("foo", 24).unwrap().unwrap(), "baz");
    assert_eq!(con.arget("foo", 25).unwrap().unwrap(), "quux");
    assert_eq!(con.arcount("foo").unwrap(), 3);
}
