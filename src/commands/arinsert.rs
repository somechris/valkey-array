//! Implementation of the `ARINSERT` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{read_write_creating_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

fn act_on_items(
    array: &mut Array,
    first_item: ValkeyString,
    arg_iter: &mut impl Iterator<Item = ValkeyString>,
) -> u64 {
    let mut last_insert_pos = array.insert(first_item);

    for item in arg_iter.by_ref() {
        last_insert_pos = array.insert(item);
    }

    last_insert_pos
}

/// Implements the `ARINSERT` command
pub fn arinsert(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    // Pulling the first item already here. So, if it's missing, we bail out early, before touching
    // the key.
    let first_item = arg_iter.next_arg()?;

    // No `err_if_further_arguments` as `act_on_items` consumes all items

    let last_insert_pos =
        read_write_creating_action!(ctx, key_name, act_on_items, first_item, &mut arg_iter);

    Ok(ValkeyValue::Integer(last_insert_pos as i64))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::commands::arinsert;
    use crate::commands::arinsert::act_on_items;
    use crate::utils::test_utils::{assert_arity_error, vkstr};
    use assertables::assert_some_eq_x;

    use crate::array::ArrayType;
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINSERT", "foo"]);

        let result = arinsert(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn act_on_items_single_item() {
        let mut array = Array::new();

        let mut iter = vec![].into_iter();

        let result = act_on_items(&mut array, vkstr("foo"), &mut iter);
        assert_eq!(result, 0); // Last item got added at position 0
        assert_some_eq_x!(array.get(0), &vkstr("foo"));
        assert_eq!(array.count(), 1);
    }

    #[test]
    fn act_on_items_multiple_items() {
        let mut array = Array::new();
        array.set_insert_cursor(42); // items will get added from position 42 onwards

        let mut iter = vec![vkstr("bar"), vkstr("baz"), vkstr("quux")].into_iter();

        let result = act_on_items(&mut array, vkstr("foo"), &mut iter);
        assert_eq!(result, 45); // Last item got added at position 45
        assert_some_eq_x!(array.get(42), &vkstr("foo"));
        assert_some_eq_x!(array.get(43), &vkstr("bar"));
        assert_some_eq_x!(array.get(44), &vkstr("baz"));
        assert_some_eq_x!(array.get(45), &vkstr("quux"));
        assert_eq!(array.count(), 4);
    }
}
