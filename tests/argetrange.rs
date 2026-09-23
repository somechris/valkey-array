//! Integration test for the `ARGETRANGE` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_position_error};
use assertables::assert_contains;
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARGETRANGE")
        .arg("foo")
        .arg("38")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARGETRANGE")
        .arg("foo")
        .arg(38)
        .arg(42)
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Wrong type for start
    let result = cmd("ARGETRANGE")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Wrong type for end
    let result = cmd("ARGETRANGE")
        .arg("foo")
        .arg("38")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.argetrange("bar", 38, 42).unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "value-37").unwrap();
    con.arset("foo", 38, "value-38").unwrap();
    con.arset("foo", 39, "value-39").unwrap();
    // Position 40 is left empty
    con.arset("foo", 41, "value-41").unwrap();
    con.arset("foo", 42, "value-42").unwrap();
    con.arset("foo", 43, "value-43").unwrap();

    // Getting from 38-42 (inclusive)
    let res = con.argetrange("foo", 38, 42).unwrap();
    assert_eq!(
        res,
        vec![
            Some("value-38".into()),
            Some("value-39".into()),
            None,
            Some("value-41".into()),
            Some("value-42".into())
        ]
    );
}

#[test]
fn reversed() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "value-37").unwrap();
    con.arset("foo", 38, "value-38").unwrap();
    con.arset("foo", 39, "value-39").unwrap();
    // Position 40 is left empty
    con.arset("foo", 41, "value-41").unwrap();
    con.arset("foo", 42, "value-42").unwrap();
    con.arset("foo", 43, "value-43").unwrap();

    // Getting from 38-42 (inclusive), but having ends reversed
    let res = con.argetrange("foo", 42, 38).unwrap();
    assert_eq!(
        res,
        vec![
            Some("value-38".into()),
            Some("value-39".into()),
            None,
            Some("value-41".into()),
            Some("value-42".into())
        ]
    );
}
