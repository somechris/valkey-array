//! Integration test for the `ARCOUNT` command

pub mod utils;

use crate::utils::ValkeyArrayTestContextBuilder;
use redis::{TypedCommands, cmd};
use redis_test::TestContextBuilder;

#[test]
fn faulty_calls() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Too few arguments
    let err = cmd("ARCOUNT").query::<u64>(&mut con).unwrap_err();
    assert!(err.to_string().contains("wrong"));

    // Too many arguments
    let err = cmd("ARCOUNT")
        .arg("foo")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert!(err.to_string().contains("wrong"));

    // Operating on non-array type
    con.set("bar", "baz").unwrap();
    let err = cmd("ARCOUNT")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap_err();
    assert_eq!(err.code().unwrap(), "WRONGTYPE");
}

#[test]
fn simple() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Checking on an unused key
    let res: u64 = cmd("ARCOUNT").arg("foo").query(&mut con).unwrap();
    assert_eq!(res, 0);

    // Adding an element
    cmd("ARSET")
        .arg("foo")
        .arg("42")
        .arg("bar")
        .query::<u64>(&mut con)
        .unwrap();

    // Now there shoud be one entry
    let res: u64 = cmd("ARCOUNT").arg("foo").query(&mut con).unwrap();
    assert_eq!(res, 1);

    // Adding two elements to the key
    cmd("ARSET")
        .arg("foo")
        .arg("23")
        .arg("baz")
        .query::<u64>(&mut con)
        .unwrap();
    cmd("ARSET")
        .arg("foo")
        .arg("4711")
        .arg("quux")
        .query::<u64>(&mut con)
        .unwrap();

    // Now there shoud be three entries
    let res: u64 = cmd("ARCOUNT").arg("foo").query(&mut con).unwrap();
    assert_eq!(res, 3);

    // Deleting an entry
    cmd("ARDEL")
        .arg("foo")
        .arg("23")
        .query::<u64>(&mut con)
        .unwrap();

    // Now there shoud be two entries
    let res: u64 = cmd("ARCOUNT").arg("foo").query(&mut con).unwrap();
    assert_eq!(res, 2);
}
