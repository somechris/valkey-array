//! Integration test for the `ARLASTITEMS` command

pub mod utils;

use crate::utils::{
    TypedArrayCommands, ValkeyArrayTestContextBuilder, assert_arity_error, assert_position_error,
    assert_unused_key_error, assert_wrong_type_error,
};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let result = cmd("ARLASTITEMS").arg("foo").query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments without `REV`
    let result = cmd("ARLASTITEMS")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Too many arguments with `REV`
    let result = cmd("ARLASTITEMS")
        .arg("foo")
        .arg(42)
        .arg("REV")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_arity_error(result);

    // Wrong type for count
    let result = cmd("ARLASTITEMS")
        .arg("foo")
        .arg("bar")
        .query::<Value>(&mut con);
    assert_position_error(result);

    // Checking on an unused key
    let result = con.arlastitems("foo", 42, false);
    assert_unused_key_error(result);

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let result = con.arlastitems("bar", 42, false);
    assert_wrong_type_error(result);
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Inserting three elements
    con.arinsert("foo", &["bar", "baz", "quux"]).unwrap();

    // Getting the last two
    let res = con.arlastitems("foo", 2, false).unwrap();
    assert_eq!(res, vec![Some("baz".to_string()), Some("quux".to_string())]);
}

#[test]
fn reverse() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Inserting three elements
    con.arinsert("foo", &["bar", "baz", "quux"]).unwrap();

    // Getting the last two reversed
    let res = con.arlastitems("foo", 2, true).unwrap();
    assert_eq!(res, vec![Some("quux".to_string()), Some("baz".to_string())]);
}

#[test]
fn mix() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Inserting three elements
    con.arinsert("foo", &["bar", "baz", "quux"]).unwrap();
    // Deleting the middle again
    con.ardel("foo", &[1]).unwrap();
    // Seeking to position 5
    con.arseek("foo", 5).unwrap();

    // Getting the more than there is, and reversed
    let res = con.arlastitems("foo", 42, true).unwrap();
    assert_eq!(
        res,
        vec![
            None,
            None,
            Some("quux".to_string()),
            None,
            Some("bar".to_string())
        ]
    );
}
