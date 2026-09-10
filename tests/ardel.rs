//! Integration test for the `ARDEL` command

pub mod utils;

use crate::utils::ValkeyArrayTestContextBuilder;
use redis::{TypedCommands, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARDEL").arg("foo").query::<u64>(&mut con).unwrap_err();
    assert!(err.to_string().contains("wrong"));

    // Too many arguments
    let err = cmd("ARDEL")
        .arg("foo")
        .arg(42)
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert!(err.to_string().contains("wrong"));

    // Wrong type for position
    let err = cmd("ARDEL")
        .arg("foo")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert!(err.to_string().contains("integer"));

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = cmd("ARDEL")
        .arg("bar")
        .arg("23")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res: u64 = cmd("ARDEL").arg("foo").arg(42).query(&mut con).unwrap();
    assert_eq!(res, 0); // 0 as nothing got deleted

    // Adding an element
    cmd("ARSET")
        .arg("foo")
        .arg("42")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap();

    // Deleting it again
    let res: u64 = cmd("ARDEL").arg("foo").arg(42).query(&mut con).unwrap();
    assert_eq!(res, 1); // 1 as an element got deleted

    // Deleting it once more
    let res: u64 = cmd("ARDEL").arg("foo").arg(42).query(&mut con).unwrap();
    assert_eq!(res, 0); // 0 as nothing got deleted
}
