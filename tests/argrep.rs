//! Integration test for the `AROP` command

extern crate core;

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
fn empty_range() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)

    // Getting used entries from 64-68 (no position in that range has a value)
    let res = con.argrep("foo", 64, 68, "EXACT", "bar").unwrap();
    assert_eq!(res, vec![]);
}

#[test]
fn exact_case_sensitive() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "bar").unwrap(); // match
    con.arset("foo", 42, "BAR").unwrap(); // no match (we're case-sensitive)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value)
    let res = con.argrep("foo", 38, 42, "EXACT", "bar").unwrap();
    assert_eq!(res, vec![38, 41]);
}

#[test]
fn exact_case_insensitive() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "bar").unwrap(); // match
    con.arset("foo", 42, "BAR").unwrap(); // match (we're case-insensitive)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value)
    let res: Vec<i64> = con
        .argrep_ex("foo", 38, 42, "EXACT", "bar", None, false, false)
        .unwrap();
    assert_eq!(res, vec![38, 41, 42]);
}

#[test]
fn match_case_sensitive() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "foobarbaz").unwrap(); // match
    con.arset("foo", 42, "fooBARbaz").unwrap(); // no match (we're case-sensitive)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value)
    let res = con.argrep("foo", 38, 42, "MATCH", "bar").unwrap();
    assert_eq!(res, vec![38, 41]);
}

#[test]
fn match_case_insensitive() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "foobarbaz").unwrap(); // match
    con.arset("foo", 42, "fooBARbaz").unwrap(); // no match (we're case-sensitive)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value)
    let res: Vec<i64> = con
        .argrep_ex("foo", 38, 42, "MATCH", "bar", None, false, false)
        .unwrap();
    assert_eq!(res, vec![38, 41, 42]);
}

#[test]
fn glob_case_sensitive() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "foobar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "foobar").unwrap(); // match
    con.arset("foo", 39, "fooooobar").unwrap(); // match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "foXbaz").unwrap(); // match
    con.arset("foo", 42, "fOoBaR").unwrap(); // no match (we're case-sensitive)
    con.arset("foo", 43, "barFOObaz").unwrap(); // no match (does not start in `f`)
    con.arset("foo", 44, "foobar").unwrap(); // ignored (not in range)

    // Getting from 38-43 (5 positions have a value)
    let res = con.argrep("foo", 38, 43, "GLOB", "fo*b").unwrap();
    assert_eq!(res, vec![38, 39, 41]);
}

#[test]
fn glob_case_insensitive() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "foobar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "foobar").unwrap(); // match
    con.arset("foo", 39, "fooooobar").unwrap(); // match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "foXbaz").unwrap(); // match
    con.arset("foo", 42, "fOoBaR").unwrap(); // no match (we're case-sensitive)
    con.arset("foo", 43, "barFOObaz").unwrap(); // no match (does not start in `f`)
    con.arset("foo", 44, "foobar").unwrap(); // ignored (not in range)

    // Getting from 38-43 (5 positions have a value)
    let res: Vec<i64> = con
        .argrep_ex("foo", 38, 43, "GLOB", "fo*b", None, false, false)
        .unwrap();
    assert_eq!(res, vec![38, 39, 41, 42]);
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
    let res: Vec<i64> = con
        .argrep_ex("foo", 38, 42, "EXACT", "bar", Some(2), true, false)
        .unwrap();
    assert_eq!(res, vec![38, 41]);
}

#[test]
fn with_values() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bar").unwrap(); // match
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "bAr").unwrap(); // match (we're case-insensitive)
    con.arset("foo", 42, "BAR").unwrap(); // match (we're case-insensitive)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 1st and 3rd match an meet limit)
    let res: Vec<(i64, String)> = con
        .argrep_ex("foo", 38, 42, "EXACT", "bar", None, false, true)
        .unwrap();
    assert_eq!(
        res,
        vec![
            (38, "bar".to_string()),
            (41, "bAr".to_string()),
            (42, "BAR".to_string())
        ]
    );
}

#[test]
fn all_options() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "bar").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "bAr").unwrap(); // match (we're case-insensitive)
    con.arset("foo", 39, "quux").unwrap(); // no match
    // Position 40 is left empty; no match
    con.arset("foo", 41, "BAR").unwrap(); // match (we're case-insensitive)
    con.arset("foo", 42, "bar").unwrap(); // ignored (we're past the limit 2)
    con.arset("foo", 43, "bar").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 1st and 3rd match an meet limit)
    let res: Vec<(i64, String)> = con
        .argrep_ex("foo", 38, 42, "EXACT", "bar", Some(2), false, true)
        .unwrap();
    assert_eq!(res, vec![(38, "bAr".to_string()), (41, "BAR".to_string()),]);
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
