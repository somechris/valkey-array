//! Integration test for the `ARLEN` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_unused_key_error,
    assert_wrong_type_error,
};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARLEN").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("ARLEN").arg("foo").arg("bar").query::<Value>(&mut con);
    assert_arity_error(result);

    // Checking on an unused key
    let result = con.arlen("foo");
    assert_unused_key_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arlen("bar");
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Highest used position is now 42
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 43);

    // Adding two elements to the key
    con.arset("foo", 23, "baz").unwrap();
    con.arset("foo", 4711, "quux").unwrap();

    // Highest used position is now 4711
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 4712);

    // Deleting an entry
    con.ardel("foo", 23).unwrap();

    // Highest used position is still 4711
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 4712);

    // Deleting the highest position
    con.ardel("foo", 4711).unwrap();

    // Highest used position fell down to 42
    let res = con.arlen("foo").unwrap();
    assert_eq!(res, 43);
}
