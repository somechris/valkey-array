//! Integration test for the `ARCOUNT` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_wrong_type_error,
};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARCOUNT").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("ARCOUNT")
        .arg("foo")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arcount("bar");
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.arcount("foo").unwrap();
    assert_eq!(res, 0);

    // Adding an element
    con.arset("foo", 42, "bar").unwrap();

    // Now there shoud be one entry
    let res = con.arcount("foo").unwrap();
    assert_eq!(res, 1);

    // Adding two elements to the key
    con.arset("foo", 23, "baz").unwrap();
    con.arset("foo", 4711, "quux").unwrap();

    // Now there shoud be three entries
    let res = con.arcount("foo").unwrap();
    assert_eq!(res, 3);

    // Deleting an entry
    con.ardel("foo", 23).unwrap();

    // Now there shoud be two entries
    let res = con.arcount("foo").unwrap();
    assert_eq!(res, 2);
}
