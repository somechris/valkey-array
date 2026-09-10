//! Integration test for the `ARDEL` command

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
    let err = cmd("ARDEL").arg("foo").query::<u64>(&mut con).unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARDEL")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Wrong type for position
    let err = cmd("ARDEL")
        .arg("foo")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.ardel("bar", 23).unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.ardel("foo", 42).unwrap();
    assert_eq!(res, 0); // 0 as nothing got deleted

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Deleting it again
    let res = con.ardel("foo", 42).unwrap();
    assert_eq!(res, 1); // 1 as an element got deleted

    // Deleting it once more
    let res = con.ardel("foo", 42).unwrap();
    assert_eq!(res, 0); // 0 as nothing got deleted
}
