//! Integration test for the `AROP` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_position_error,
};
use assertables::assert_contains;
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("AROP")
        .arg("foo")
        .arg("23")
        .arg("42")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("AROP")
        .arg("foo")
        .arg("23")
        .arg("42")
        .arg("USED")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Wrong type for start
    let result = cmd("AROP")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .arg("USED")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Wrong type for end
    let result = cmd("AROP")
        .arg("foo")
        .arg("23")
        .arg("bar")
        .arg("USED")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Unknown operation
    let err = cmd("AROP")
        .arg("foo")
        .arg("23")
        .arg("42")
        .arg("BAZ")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "operation");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arop::<Value>("bar", 38, 42, "USED").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn used_simple() {
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

    // Getting used entries from 64--68 (no position in that range has a value)
    let res: u64 = con.arop("foo", 64, 68, "USED").unwrap();
    assert_eq!(res, 0);

    // Getting from 38-42 (4 positions have a value)
    let res: u64 = con.arop("foo", 38, 42, "USED").unwrap();
    assert_eq!(res, 4);
}

#[test]
fn max_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "11").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "42").unwrap(); // initial max
    con.arset("foo", 39, "value-380").unwrap(); // not considered (not a number)
    // Position 40 is left empty, hence not considered
    con.arset("foo", 41, "4096").unwrap(); // final max
    con.arset("foo", 42, "-4.2").unwrap(); // below max (float)
    con.arset("foo", 43, "23").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 3 contribute)
    let res: String = con.arop("foo", 38, 42, "MAX").unwrap();
    assert_eq!(res, "4096");
}

#[test]
fn min_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "11").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "23").unwrap(); // initial min
    con.arset("foo", 39, "value-38").unwrap(); // not considered (not a number)
    // Position 40 is left empty, hence not considered
    con.arset("foo", 41, "-471.1").unwrap(); // final min
    con.arset("foo", 42, "-4.2").unwrap(); // above min
    con.arset("foo", 43, "4096").unwrap(); // not summed (not in range)

    // Getting from 38-42 (4 positions have a value, 3 contribute)
    let res: String = con.arop("foo", 38, 42, "MIN").unwrap();
    assert_eq!(res, "-471.1");
}

#[test]
fn and_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "11").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "7.42").unwrap(); // ANDs initial value to 7
    con.arset("foo", 39, "value-38").unwrap(); // not considered (not a number)
    // Position 40 is left empty, hence not considered
    con.arset("foo", 41, "5").unwrap(); // ANDs 7 to 5
    con.arset("foo", 42, "-2").unwrap(); // ANDs 5 to 4
    con.arset("foo", 43, "4096").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 3 contribute)
    let res: String = con.arop("foo", 38, 42, "AND").unwrap();
    assert_eq!(res, "4");
}

#[test]
fn or_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "11").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "1.42").unwrap(); // ORs inital value to 1
    con.arset("foo", 39, "value-38").unwrap(); // not considered (not a number)
    // Position 40 is left empty, hence not considered
    con.arset("foo", 41, "3").unwrap(); // ORs 1 to 3
    con.arset("foo", 42, "-8").unwrap(); // ORs 3 to -5
    con.arset("foo", 43, "4096").unwrap(); // not summed (not in range)

    // Getting from 38-42 (4 positions have a value, 3 contribute)
    let res: String = con.arop("foo", 38, 42, "OR").unwrap();
    assert_eq!(res, "-5");
}

#[test]
fn xor_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "11").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "1.42").unwrap(); // XORs inital value to 1
    con.arset("foo", 39, "value-38").unwrap(); // not considered (not a number)
    // Position 40 is left empty, hence not considered
    con.arset("foo", 41, "3").unwrap(); // XORs 1 to 2
    con.arset("foo", 42, "-8").unwrap(); // XORs 2 to -6
    con.arset("foo", 43, "4096").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 3 contribute)
    let res: String = con.arop("foo", 38, 42, "XOR").unwrap();
    assert_eq!(res, "-6");
}

#[test]
fn sum_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "11").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "23").unwrap(); // summed (integer)
    con.arset("foo", 39, "value-38").unwrap(); // not summed (not a number)
    // Position 40 is left empty, hence not summed
    con.arset("foo", 41, "-42").unwrap(); // summed (negative)
    con.arset("foo", 42, "-.4711").unwrap(); // summed (float)
    con.arset("foo", 43, "4096").unwrap(); // ignored (not in range)

    // Getting from 38-42 (3 positions have a value)
    let res: String = con.arop("foo", 38, 42, "SUM").unwrap();
    assert_eq!(res, "-19.4711");
}

#[test]
fn match_simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "foo").unwrap(); // ignored (not in range)
    con.arset("foo", 38, "foo").unwrap(); // matched
    con.arset("foo", 39, "bar").unwrap(); // not matched (different text)
    // Position 40 is left empty, hence not considered
    con.arset("foo", 41, "  foo  ").unwrap(); // not matched (extra whitespace)
    con.arset("foo", 42, "foo").unwrap(); // matched
    con.arset("foo", 43, "foo").unwrap(); // ignored (not in range)

    // Getting from 38-42 (4 positions have a value, 2 match)
    let res: String = con.arop_ex("foo", 38, 42, "MATCH", "foo").unwrap();
    assert_eq!(res, "2");
}

#[test]
fn reverse() {
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

    // Getting from 38-42 (4 positions have a value), but start/end are reversed
    let res: u64 = con.arop("foo", 42, 38, "USED").unwrap();
    assert_eq!(res, 4);
}
