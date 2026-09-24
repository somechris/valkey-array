//! Integration test for the `ARMSET` command

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

    // Wrong type for first position
    let result = cmd("ARMSET")
        .arg("foo")
        .arg("bar")
        .arg("baz")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Wrong type for second position
    let result = cmd("ARMSET")
        .arg("foo")
        .arg("38")
        .arg("bar")
        .arg("quux")
        .arg("quuux")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.armset("bar", &[(23, "foo")]);
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Add a few elements
    con.arset("foo", 23, "value-23-1").unwrap();

    // Setting 42 (new), 4711 (new), 23 (was set above), 42 (was set in first argument)
    let res = con
        .armset(
            "foo",
            &[
                (42, "value-42-1"),
                (4711, "value-4711"),
                (23, "value-23-2"),
                (42, "value-42-2"),
            ],
        )
        .unwrap();
    assert_eq!(res, 2); // 42 occurs twice, and 23 has been set before. So only 2 new slots

    // Checking the slots have expected values
    assert_eq!(con.arget("foo", 23).unwrap().unwrap(), "value-23-2");
    assert_eq!(con.arget("foo", 42).unwrap().unwrap(), "value-42-2");
    assert_eq!(con.arget("foo", 4711).unwrap().unwrap(), "value-4711");
}
