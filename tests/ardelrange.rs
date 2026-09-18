//! Integration test for the `ARDELRANGE` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::{assert_contains, assert_none};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARDELRANGE")
        .arg("foo")
        .arg("23")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // No "too many args" check, as `ARDELRANGE` consumes all the items that are there.

    // Wrong type for start
    let err = cmd("ARDELRANGE")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Wrong type for end
    let err = cmd("ARDELRANGE")
        .arg("foo")
        .arg("23")
        .arg("bar")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.ardelrange("bar", &[(23, 42)]).unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 22, "value-22").unwrap(); // will be kept
    con.arset("foo", 23, "value-23").unwrap(); // will get deleted
    con.arset("foo", 24, "value-24").unwrap(); // will get deleted
    con.arset("foo", 41, "value-41").unwrap(); // will get deleted
    con.arset("foo", 42, "value-42").unwrap(); // will get deleted
    con.arset("foo", 43, "value-43").unwrap(); // will be kept

    // Deleting from 23-42 (inclusive)
    let res = con.ardelrange("foo", &[(23, 42)]).unwrap();
    assert_eq!(res, 4); // 4 elements got deleted

    con.arget("foo", 22).unwrap().unwrap(); // was kept
    assert_none!(con.arget("foo", 23).unwrap()); // got deleted
    assert_none!(con.arget("foo", 24).unwrap()); // got deleted
    assert_none!(con.arget("foo", 41).unwrap()); // got deleted
    assert_none!(con.arget("foo", 42).unwrap()); // got deleted
    con.arget("foo", 43).unwrap().unwrap(); // was kept
}

#[test]
fn multiple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 22, "value-22").unwrap(); // will be kept
    con.arset("foo", 23, "value-23").unwrap(); // will get deleted
    con.arset("foo", 24, "value-24").unwrap(); // will get deleted
    con.arset("foo", 41, "value-41").unwrap(); // will get deleted
    con.arset("foo", 42, "value-42").unwrap(); // will get deleted
    con.arset("foo", 43, "value-43").unwrap(); // will be kept
    con.arset("foo", 44, "value-44").unwrap(); // will be kept

    // Deleting from 23-42 (inclusive)
    let res = con
        .ardelrange("foo", &[(42, 43), (43, 44), (20, 23)])
        .unwrap();
    assert_eq!(res, 5); // 5 elements got deleted

    con.arget("foo", 24).unwrap().unwrap(); // was kept
    con.arget("foo", 41).unwrap().unwrap(); // was kept
    assert_eq!(con.arcount("foo").unwrap(), 2);
}

#[test]
fn reversed() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 22, "value-22").unwrap(); // will be kept
    con.arset("foo", 23, "value-23").unwrap(); // will get deleted
    con.arset("foo", 24, "value-24").unwrap(); // will get deleted
    con.arset("foo", 41, "value-41").unwrap(); // will get deleted
    con.arset("foo", 42, "value-42").unwrap(); // will get deleted
    con.arset("foo", 43, "value-43").unwrap(); // will be kept

    // Deleting from 23-42 (inclusive), but give them reversed
    let res = con.ardelrange("foo", &[(42, 23)]).unwrap();
    assert_eq!(res, 4); // 4 elements got deleted

    con.arget("foo", 22).unwrap().unwrap(); // was kept
    assert_none!(con.arget("foo", 23).unwrap()); // got deleted
    assert_none!(con.arget("foo", 24).unwrap()); // got deleted
    assert_none!(con.arget("foo", 41).unwrap()); // got deleted
    assert_none!(con.arget("foo", 42).unwrap()); // got deleted
    con.arget("foo", 43).unwrap().unwrap(); // was kept
}
