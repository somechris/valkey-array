//! Implementation of the `ARDELRANGE` command

use crate::Array;
use crate::array::{ArrayType, Range};
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_write_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_ranges(array: &mut Array, ranges: Vec<Range>) -> u64 {
    let mut count = 0;

    for (start, end) in ranges {
        count += (start..=end).map(|pos| array.del(pos)).sum::<u64>();
    }
    count
}

/// Implements the `ARDELRANGE` command
pub fn ardelrange(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    let mut arg_iter = arg_iter.peekable();
    let mut ranges = Vec::new();
    while arg_iter.peek().is_some() {
        ranges.push(arg_iter.next_range()?);
    }

    err_if_further_arguments(arg_iter)?;

    let count = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Integer(0),
        act_on_ranges,
        ranges
    );

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::array::ArrayType;
    use crate::commands::ardelrange;
    use crate::commands::ardelrange::act_on_ranges;
    use crate::test_utils::{assert_arity_error, assert_position_error, vkstr};
    use assertables::assert_some_eq_x;
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23"]);

        let result = ardelrange(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn wrong_argument_type_first_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "bar", "42"]);

        let result = ardelrange(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_first_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23", "bar"]);

        let result = ardelrange(&ctx, args);
        assert_position_error(result);
    }
    #[test]
    fn wrong_argument_type_later_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23", "42", "bar", "4711"]);

        let result = ardelrange(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_later_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23", "42", "151", "bar"]);

        let result = ardelrange(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_non_pair() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23", "42", "151"]);

        let result = ardelrange(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn act_on_ranges_empty() {
        let mut array = Array::new();
        array.set(42, vkstr("foo"));

        let count = act_on_ranges(&mut array, vec![]);
        assert_eq!(count, 0); // No item was removed
        assert_some_eq_x!(array.get(42), &vkstr("foo"));
        assert_eq!(array.count(), 1);
    }

    #[test]
    fn act_on_ranges_single_range() {
        let mut array = Array::new();
        array.set(23, vkstr("foo"));
        array.set(42, vkstr("bar"));
        array.set(43, vkstr("baz"));
        array.set(44, vkstr("quux"));

        let count = act_on_ranges(&mut array, vec![(40, 43)]);
        assert_eq!(count, 2); // 42, and 43 was removed
        assert_some_eq_x!(array.get(23), &vkstr("foo"));
        assert_some_eq_x!(array.get(44), &vkstr("quux"));
        assert_eq!(array.count(), 2);
    }

    #[test]
    fn act_on_ranges_multiple_ranges() {
        let mut array = Array::new();
        array.set(23, vkstr("foo"));
        array.set(42, vkstr("bar"));
        array.set(43, vkstr("baz"));
        array.set(44, vkstr("quux"));
        array.set(4711, vkstr("quux"));

        // 4711 is deleted once, 23 twice, and 151 does not exist
        let count = act_on_ranges(&mut array, vec![(40, 42), (43, 45), (4710, 4712)]);
        assert_eq!(count, 4); // 42, 43, 44, and 4711 got removed
        assert_some_eq_x!(array.get(23), &vkstr("foo"));
        assert_eq!(array.count(), 1);
    }
}
