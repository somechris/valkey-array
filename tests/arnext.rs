//! Integration test for the `ARNEXT` command

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
    let err = cmd("ARNEXT").query::<Value>(&mut con).unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARNEXT")
        .arg("foo")
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arnext("bar").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 0);

    // Adding an element
    con.arinsert("foo", "bar").unwrap();

    // Next insert position should have advanced
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 1);

    // Deleting does not change the cursor
    con.ardel("foo", 0).unwrap();
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 1);

    // Inserting at higher position does not change the cursor
    con.arset("foo", 42, "baz").unwrap();
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 1);

    // Adding an element to check that we're not stuck at 1
    con.arinsert("foo", "bar").unwrap();
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 2);
}
