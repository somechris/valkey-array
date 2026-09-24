//! Integration test for the `ARMGET` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_position_error,
    assert_wrong_type_error,
};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Bootstrapping the array
    con.arset("foo", 1, "baz").unwrap();

    // Wrong type for first position
    let result = cmd("ARMGET")
        .arg("foo")
        .arg("bar")
        .arg("42")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Wrong type for second position
    let result = cmd("ARMGET")
        .arg("foo")
        .arg("38")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.armget("bar", &[23, 42]);
    assert_wrong_type_error(result);
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
