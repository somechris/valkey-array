//! Integration test for the `ARLEN` command

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
    let err = cmd("ARLEN").query::<u64>(&mut con).unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARLEN")
        .arg("foo")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arlen("bar").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 0);

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Highest used position is now 42
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 43);

    // Adding two elements to the key
    con.arset("foo", 23, "baz").unwrap();
    con.arset("foo", 4711, "quux").unwrap();

    // Highest used position is now 4711
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 4712);

    // Deleting an entry
    con.ardel("foo", 23).unwrap();

    // Highest used position is still 4711
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 4712);

    // Deleting the highest position
    con.ardel("foo", 4711).unwrap();

    // Highest used position fell down to 42
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 43);
}
