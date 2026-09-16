//! Integration test for the `AROP` command

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
    let err = cmd("ARGREP")
        .arg("foo")
        .arg("23")
        .arg("42")
        .arg("EXACT")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Too many arguments
    let err = cmd("ARGREP")
        .arg("foo")
        .arg("23")
        .arg("42")
        .arg("EXACT")
        .arg("bar")
        .arg("baz")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // Wrong type for start
    let err = cmd("ARGREP")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .arg("EXACT")
        .arg("baz")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Wrong type for end
    let err = cmd("ARGREP")
        .arg("foo")
        .arg("23")
        .arg("bar")
        .arg("EXACT")
        .arg("baz")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Unknown operation
    let err = cmd("ARGREP")
        .arg("foo")
        .arg("23")
        .arg("42")
        .arg("BAZ")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "operation");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.argrep("bar", 38, 42, "EXACT", "baz").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn exact_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "bar").unwrap(); // match
    con.arset("foo", 42, "  bar  ").unwrap(); // no match (extra whitespace)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting used entries from 64--68 (no position in that range has a value)
    let res = con.argrep("foo", 64, 68, "EXACT", "bar").unwrap();
    assert_eq!(res, vec![]);

    // Getting from 38-42 (4 positions have a value)
    let res = con.argrep("foo", 38, 42, "EXACT", "bar").unwrap();
    assert_eq!(res, vec![38, 41]);
}

#[test]
fn limit() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "bar").unwrap(); // match. Reaches limit 2
    con.arset("foo", 42, "bar").unwrap(); // no match (extra whitespace)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 1st and 3rd match an meet limit)
    let res = con.argrep_ex("foo", 38, 42, "EXACT", "bar", 2).unwrap();
    assert_eq!(res, vec![38, 41]);
}

#[test]
fn reverse() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "bar").unwrap(); // match
    con.arset("foo", 42, "quuux").unwrap(); // no match
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value), but start/end are reversed
    let res = con.argrep("foo", 42, 38, "EXACT", "bar").unwrap();
    assert_eq!(res, vec![38, 41]);
}
