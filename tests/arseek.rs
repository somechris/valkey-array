//! Integration test for the `ARSEEK` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_position_error,
    assert_wrong_type_error,
};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARSEEK").arg("foo").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("ARSEEK")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Wrong type for position
    let result = cmd("ARSEEK").arg("foo").arg("bar").query::<Value>(&mut con);
    assert_position_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arseek("bar", 23);
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.arseek("foo", 42).unwrap();
    assert_eq!(res, 0);

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Setting the insert position
    let res = con.arseek("foo", 23).unwrap();
    assert_eq!(res, 1);

    // Inserting, and then checking the expected position's contents
    con.arinsert("foo", "baz").unwrap();
    let res = con.arget("foo", 23).unwrap();
    assert_eq!(res.unwrap(), "baz");
}
