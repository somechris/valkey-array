//! Implementation of the `ARRING` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{NextArgExtras, read_write_creating_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

fn act_on_items(
    array: &mut Array,
    buffer_size: u64,
    first_item: ValkeyString,
    arg_iter: &mut impl Iterator<Item = ValkeyString>,
) -> u64 {
    let mut last_insert_pos = array.insert_ring(buffer_size, first_item);

    for item in arg_iter.by_ref() {
        last_insert_pos = array.insert_ring(buffer_size, item);
    }

    last_insert_pos
}

/// Implements the `ARRING` command
pub fn arring(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    // As the buffer size is the array position where it wraps over, we parse as position
    let buffer_size = arg_iter.next_position()?;
    // Pulling the first item already here. So, if it's missing, we bail out early, before touching
    // the key.
    let first_item = arg_iter.next_arg()?;

    // No `err_if_further_arguments` as `act_on_items` consumes all items

    let last_inserted_position = read_write_creating_action!(
        ctx,
        key_name,
        act_on_items,
        buffer_size,
        first_item,
        &mut arg_iter
    );

    Ok(ValkeyValue::Integer(last_inserted_position as i64))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::array::ArrayType;
    use crate::commands::arring;
    use crate::commands::arring::act_on_items;
    use crate::test_utils::{assert_arity_error, assert_position_error, vkstr};
    use assertables::assert_some_eq_x;
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARRING", "foo", "23"]);

        let result = arring(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARRING", "foo", "bar", "baz"]);

        let result = arring(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn act_on_items_single_item() {
        let mut array = Array::new();

        let mut iter = vec![].into_iter();

        let result = act_on_items(&mut array, 42, vkstr("foo"), &mut iter);
        assert_eq!(result, 0); // Last item got added at position 0
        assert_some_eq_x!(array.get(0), &vkstr("foo"));
        assert_eq!(array.count(), 1);
    }

    #[test]
    fn act_on_items_multiple_items() {
        let mut array = Array::new();
        array.set_insert_cursor(4711); // items will get added from 7 (= 4711 % 42) onwards
        array.set(8, vkstr("QUUUX")); // Will get overwritten (not jumped over)

        let mut iter = vec![vkstr("bar"), vkstr("baz"), vkstr("quux")].into_iter();

        let result = act_on_items(&mut array, 42, vkstr("foo"), &mut iter);
        assert_eq!(result, 10); // Last item got added at position 10
        assert_some_eq_x!(array.get(7), &vkstr("foo"));
        assert_some_eq_x!(array.get(8), &vkstr("bar"));
        assert_some_eq_x!(array.get(9), &vkstr("baz"));
        assert_some_eq_x!(array.get(10), &vkstr("quux"));
        assert_eq!(array.count(), 4);
    }
}
