//! Integration test for the `ARRING` command

pub mod utils;

use crate::utils::{TypedArrayCommands, ValkeyArrayTestContextBuilder};
use assertables::{assert_contains, assert_none};
use redis::{TypedCommands, Value, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARRING")
        .arg("foo")
        .arg(42)
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "wrong");

    // No "too many args" check, as `ARRING` consumes all the items that are there.

    // Wrong type for position
    let err = cmd("ARRING")
        .arg("foo")
        .arg("bar")
        .arg("baz")
        .query::<Value>(&mut con)
        .unwrap_err();
    assert_contains!(err.to_string(), "integer");

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = con.arring("bar", 23, "quux").unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // We test with a ring buffer of size three. So the fourth element should overwrite the
    // first.

    // Insert first item
    let res = con.arring("foo", 3, "bar").unwrap();
    assert_eq!(res, 0);

    // Insert second item
    let res = con.arring("foo", 3, "baz").unwrap();
    assert_eq!(res, 1);

    // Insert third item
    let res = con.arring("foo", 3, "quux").unwrap();
    assert_eq!(res, 2);

    // Insert fourth item. This should overrun and get written to position 0
    let res = con.arring("foo", 3, "quuux").unwrap();
    assert_eq!(res, 0);

    // Check array contents
    assert_eq!(con.arget("foo", 0).unwrap().unwrap(), "quuux");
    assert_eq!(con.arget("foo", 1).unwrap().unwrap(), "baz");
    assert_eq!(con.arget("foo", 2).unwrap().unwrap(), "quux");
    assert_none!(con.arget("foo", 3).unwrap());
}

#[test]
fn multiple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // We test with a ring buffer of size five. So it should go from position 5 back to 0.

    // Add a value that will get overwritten by the below `arring`
    con.arset("foo", 3, "value-3").unwrap();

    // Set the insert cursor to not start at 0
    con.arseek("foo", 42).unwrap(); // 42 % 5 = 2. So first ring insert is at 2.

    let res = con
        .arring("foo", 5, &["bar", "baz", "quux", "quuux"])
        .unwrap();
    assert_eq!(res, 0); // Last insert was at position 0

    // Check array contents
    assert_eq!(con.arget("foo", 0).unwrap().unwrap(), "quuux");
    assert_none!(con.arget("foo", 1).unwrap());
    assert_eq!(con.arget("foo", 2).unwrap().unwrap(), "bar");
    assert_eq!(con.arget("foo", 3).unwrap().unwrap(), "baz");
    assert_eq!(con.arget("foo", 4).unwrap().unwrap(), "quux");
    assert_none!(con.arget("foo", 5).unwrap());
    assert_none!(con.arget("foo", 6).unwrap());
}
