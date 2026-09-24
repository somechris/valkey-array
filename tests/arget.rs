//! Integration test for the `ARGET` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_position_error,
    assert_unused_key_error, assert_wrong_type_error,
};
use assertables::{assert_none, assert_some_eq_x};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARGET").arg("foo").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("ARGET")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Wrong type for position
    let result = cmd("ARGET").arg("foo").arg("bar").query::<Value>(&mut con);
    assert_position_error(result);

    // Getting from an unused key
    let result = con.arget("foo", 42);
    assert_unused_key_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arget("bar", 23);
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Getting the added element
    let res = con.arget("foo", 42).unwrap();
    assert_some_eq_x!(res, "bar");

    // Getting another (non-existing) element from the key
    let res = con.arget("foo", 23).unwrap();
    assert_none!(res);
}
