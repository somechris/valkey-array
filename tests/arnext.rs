//! Integration test for the `ARNEXT` command

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
    let result = cmd("ARNEXT").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments
    let result = cmd("ARNEXT").arg("foo").arg("bar").query::<Value>(&mut con);
    assert_arity_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arnext("bar");
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 0);

    // Adding an element
    con.arinsert("foo", "bar").unwrap();

    // Next insert position should have advanced
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 1);

    // Deleting does not change the cursor
    con.ardel("foo", 0).unwrap();
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 1);

    // Inserting at higher position does not change the cursor
    con.arset("foo", 42, "baz").unwrap();
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 1);

    // Adding an element to check that we're not stuck at 1
    con.arinsert("foo", "bar").unwrap();
    let res = con.arnext("foo").unwrap();
    assert_eq!(res, 2);
}
