//! Implementation of the `ARSET` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{read_write_creating_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

fn act_on_items(
    array: &mut Array,
    mut position: u64,
    arg_iter: &mut impl Iterator<Item = ValkeyString>,
) -> u64 {
    let mut count = 0;

    for item in arg_iter {
        count += array.set(position, item);
        position += 1;
    }

    count
}

/// Implements the `ARSET` command
pub fn arset(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let position = arg_iter.next_u64()?;

    // No `err_if_further_arguments` as `act_on_items` consumes all items

    let count = read_write_creating_action!(ctx, key_name, act_on_items, position, &mut arg_iter);

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::array::ArrayType;
    use crate::commands::arset;
    use crate::commands::arset::act_on_items;
    use crate::test_utils::vkstr;
    use assertables::{assert_contains, assert_none, assert_some_eq_x};
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "bar", "baz"]);

        let result = arset(&ctx, args);
        let err = result.expect_err("ARSET should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn act_on_items_no_item() {
        let mut array = Array::new();

        let mut iter = vec![].into_iter();

        let result = act_on_items(&mut array, 42, &mut iter);
        assert_eq!(result, 0); // No new slot got taken
        assert_none!(array.get(0));
        assert_none!(array.get(42));
        assert_eq!(array.count(), 0);
    }

    #[test]
    fn act_on_items_single_item() {
        let mut array = Array::new();

        let mut iter = vec![vkstr("foo")].into_iter();

        let result = act_on_items(&mut array, 42, &mut iter);
        assert_eq!(result, 1); // One new slot got taken
        assert_none!(array.get(0));
        assert_some_eq_x!(array.get(42), &vkstr("foo"));
        assert_eq!(array.count(), 1);
    }

    #[test]
    fn act_on_items_multiple_items() {
        let mut array = Array::new();
        array.set_insert_cursor(42); // items will get added from position 42 onwards
        array.set(24, vkstr("value-24")); // This will get overwritten below

        let mut iter = vec![vkstr("bar"), vkstr("baz"), vkstr("quux")].into_iter();

        let result = act_on_items(&mut array, 23, &mut iter);
        assert_eq!(result, 2); // Two new slots got taken (24 was occupied before)
        assert_some_eq_x!(array.get(23), &vkstr("bar"));
        assert_some_eq_x!(array.get(24), &vkstr("baz"));
        assert_some_eq_x!(array.get(25), &vkstr("quux"));
        assert_none!(array.get(0));
        assert_none!(array.get(42));
        assert_eq!(array.count(), 3);
    }
}
