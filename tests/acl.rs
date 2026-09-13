//! Integration test for ACLs

pub mod utils;

use crate::utils::ValkeyArrayTestContextBuilder;
use assertables::assert_contains;
use redis::TypedCommands;
use redis_test::TestContextBuilder;
use std::collections::HashSet;

#[test]
fn basic() {
    let ctx = TestContextBuilder::build_for_valkey_array();
    let mut con = ctx.connection();

    // Check that the `array` ACL is present
    let acls = con.acl_cat().unwrap();
    assert_contains!(acls, "array");

    // Check that all the relevant commands are in the `array` ACL group
    let array_commands = con.acl_cat_categoryname("array").unwrap();
    let expected = HashSet::from([
        "arcount".to_string(),
        "ardel".to_string(),
        "ardelrange".to_string(),
        "arget".to_string(),
        "argetrange".to_string(),
        "arinfo".to_string(),
        "arinsert".to_string(),
        "arlen".to_string(),
        "arnext".to_string(),
        "arring".to_string(),
        "arseek".to_string(),
        "arset".to_string(),
    ]);
    assert_eq!(array_commands, expected);
}
