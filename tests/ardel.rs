//! Integration test for the `ARDEL` command

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
    let result = cmd("ARDEL").query::<Value>(&mut con);
    assert_arity_error(result);

    // No "too many args" check, as `ARDEL` consumes all the items that are there.

    // Wrong type for first position
    let result = cmd("ARDEL").arg("foo").arg("bar").query::<Value>(&mut con);
    assert_position_error(result);

    // Wrong type for later position
    let result = cmd("ARDEL")
        .arg("foo")
        .arg("42")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.ardel("bar", 23);
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.ardel("foo", 42).unwrap();
    assert_eq!(res, 0); // 0 as nothing got deleted

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Deleting it again
    let res = con.ardel("foo", 42).unwrap();
    assert_eq!(res, 1); // 1 as an element got deleted

    // Deleting it once more
    let res = con.ardel("foo", 42).unwrap();
    assert_eq!(res, 0); // 0 as nothing got deleted

    // Adding more element
    con.arset("foo", 42, "bar").unwrap(); // will get deleted twice
    con.arset("foo", 43, "baz").unwrap(); // will get deleted once
    con.arset("foo", 44, "quux").unwrap(); // will not get deleted

    // Deleting some again (41 is empty, 42 gets deleted twice, 43 once)
    let res = con.ardel("foo", &[41, 42, 43, 42]).unwrap();
    assert_eq!(res, 2); // 42 and 43 were effectively deleted
}
