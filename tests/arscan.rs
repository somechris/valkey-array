//! Integration test for the `ARSCAN` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_position_error,
    assert_wrong_type_error,
};
use assertables::assert_is_empty;
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARSCAN").arg("foo").arg("38").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("ARSCAN")
        .arg("foo")
        .arg(38)
        .arg(42)
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Wrong type for start
    let result = cmd("ARSCAN")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Wrong type for end
    let result = cmd("ARSCAN")
        .arg("foo")
        .arg("38")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arscan("bar", 23, 46);
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "value-37").unwrap();
    con.arset("foo", 38, "value-38").unwrap();
    // Position 39 is left empty
    con.arset("foo", 40, "value-40").unwrap();
    // Position 40 is left empty
    // Position 41 is left empty
    con.arset("foo", 42, "value-42").unwrap();
    con.arset("foo", 43, "value-43").unwrap();

    // Getting from 23-42 (inclusive)
    let res = con.arscan("foo", 23, 42).unwrap();
    assert_eq!(
        res,
        vec![
            (37, "value-37".to_string()),
            (38, "value-38".to_string()),
            (40, "value-40".to_string()),
            (42, "value-42".to_string()),
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
    // Position 39 is left empty
    con.arset("foo", 40, "value-40").unwrap();
    // Position 40 is left empty
    // Position 41 is left empty
    con.arset("foo", 42, "value-42").unwrap();
    con.arset("foo", 43, "value-43").unwrap();

    // Getting from 23-42 (inclusive), but having ends reversed
    let res = con.arscan("foo", 42, 23).unwrap();
    assert_eq!(
        res,
        vec![
            (37, "value-37".to_string()),
            (38, "value-38".to_string()),
            (40, "value-40".to_string()),
            (42, "value-42".to_string()),
        ]
    );
}

#[test]
fn limited() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 37, "value-37").unwrap();
    con.arset("foo", 38, "value-38").unwrap();
    // Position 39 is left empty
    con.arset("foo", 40, "value-40").unwrap();
    // Position 40 is left empty
    // Position 41 is left empty
    con.arset("foo", 42, "value-42").unwrap();
    con.arset("foo", 43, "value-43").unwrap();

    // Getting from 23-42 (inclusive), limit 0
    let res = con.arscan_limited("foo", 23, 42, 0).unwrap();
    assert_is_empty!(res);

    // Getting from 23-42 (inclusive), limit 3
    let res = con.arscan_limited("foo", 23, 42, 3).unwrap();
    assert_eq!(
        res,
        vec![
            (37, "value-37".to_string()),
            (38, "value-38".to_string()),
            (40, "value-40".to_string()),
        ]
    );
}
