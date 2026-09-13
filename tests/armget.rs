//! Integration test for the `ARMGET` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::assert_contains;
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Bootstrapping the array
    con.arset("foo", 1, "baz").unwrap();

    // Wrong type for first position
    let err = cmd("ARMGET")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Wrong type for second position
    let err = cmd("ARMGET")
        .arg("foo")
        .arg("38")
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.armget("bar", &[23, 42]).unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 23, "value-23").unwrap();
    con.arset("foo", 42, "value-42").unwrap();

    // Getting from 42 (exists), 4711 (missing), 23 (exists)
    let res = con.armget("foo", &[42, 4711, 23]).unwrap();
    assert_eq!(
        res,
        vec![Some("value-42".into()), None, Some("value-23".into())]
    );
}
