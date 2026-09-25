//! Implementation of the `ARDEL` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{ValkeyStringExtras, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

fn act_on_positions(array: &mut Array, positions: Vec<u64>) -> u64 {
    positions.into_iter().map(|pos| array.del(pos)).sum()
}

/// Implements the `ARDEL` command
pub fn ardel(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    let positions = arg_iter
        .map(|str| str.parse_position())
        .collect::<ValkeyResult<Vec<u64>>>()?;

    // No `err_if_further_arguments` as the above iteration already consumed all items

    let count = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Integer(0),
        act_on_positions,
        positions
    );

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::array::ArrayType;
    use crate::commands::ardel;
    use crate::commands::ardel::act_on_positions;
    use crate::utils::test_utils::{assert_position_error, vkstr};
    use assertables::assert_some_eq_x;
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn wrong_argument_type_first() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDEL", "foo", "bar"]);

        let result = ardel(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_later() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDEL", "foo", "42", "bar"]);

        let result = ardel(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn act_on_positions_empty() {
        let mut array = Array::new();
        array.set(42, vkstr("foo"));

        let count = act_on_positions(&mut array, vec![]);
        assert_eq!(count, 0); // No item was removed
        assert_some_eq_x!(array.get(42), &vkstr("foo"));
        assert_eq!(array.count(), 1);
    }

    #[test]
    fn act_on_positions_single_item() {
        let mut array = Array::new();
        array.set(23, vkstr("foo"));
        array.set(42, vkstr("bar"));

        let count = act_on_positions(&mut array, vec![23]);
        assert_eq!(count, 1); // 1 item was removed
        assert_some_eq_x!(array.get(42), &vkstr("bar"));
        assert_eq!(array.count(), 1);
    }

    #[test]
    fn act_on_positions_multiple_items() {
        let mut array = Array::new();
        array.set(23, vkstr("foo"));
        array.set(42, vkstr("bar"));
        array.set(4711, vkstr("bar"));

        // 4711 is deleted once, 23 twice, and 151 does not exist
        let count = act_on_positions(&mut array, vec![4711, 23, 151, 23]);
        assert_eq!(count, 2); // 23 and 4711 got removed
        assert_some_eq_x!(array.get(42), &vkstr("bar"));
        assert_eq!(array.count(), 1);
    }
}
