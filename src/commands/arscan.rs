//! Implementation of the `ARSCAN` command

use crate::Array;
use crate::array::{ArrayType, Range};
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_only_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(array: &Array, range: Range, opt_limit: Option<u64>) -> Vec<ValkeyValue> {
    let (limited, limit) = match opt_limit {
        Some(limit) => {
            if limit == 0 {
                // Nothing to do
                return vec![];
            }
            (true, limit as usize)
        }
        None => (false, 0),
    };

    let mut ret = vec![];
    for (position, maybe_value) in array.range_iter(range) {
        if let Some(value) = maybe_value {
            ret.push((vec![ValkeyValue::from(position as i64), value.into()]).into());
        }

        // Checking an eventual limit
        if limited && ret.len() == limit {
            break;
        }
    }
    ret
}

/// Implements the `ARSCAN` command
pub fn arscan(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let range = arg_iter.next_range()?;
    let mut opt_limit = None;

    // Parse optional limit
    if let Some(arg) = arg_iter.next() {
        match arg.to_string().to_ascii_uppercase().as_str() {
            "LIMIT" => opt_limit = Some(arg_iter.next_u64()?),
            _ => return Err(ValkeyError::WrongArity),
        }
        err_if_further_arguments(arg_iter)?;
    }

    let items = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        range,
        opt_limit
    );

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::array::{ArrayType, Range};
    use crate::commands::arscan;
    use crate::commands::arscan::act_on_range;
    use crate::utils::test_utils::{
        assert_arity_error, assert_position_error, u32s_with_vals_to_vec_value, vkstr,
    };
    use assertables::{assert_contains, assert_is_empty};
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23"]);

        let result = arscan(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high_without_limit() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "42", "bar"]);

        let result = arscan(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high_with_limit() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "42", "LIMIT", "4711", "bar"]);

        let result = arscan(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "bar", "42"]);

        let result = arscan(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "bar"]);

        let result = arscan(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_limit() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "42", "LIMIT", "bar"]);

        let result = arscan(&ctx, args);
        let err = result.expect_err("ARSCAN should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn act_on_range_only_empty_positions() {
        let array = Array::new();

        for limit in [None, Some(0), Some(1)] {
            let result = act_on_range(&array, Range::new(23, 42), limit);
            assert_is_empty!(result);
        }
    }

    #[test]
    fn act_on_range_single_position() {
        let mut array = Array::new();
        array.set(42, vkstr("foo"));

        // No limit
        let result = act_on_range(&array, Range::new(41, 42), None);
        assert_eq!(result, u32s_with_vals_to_vec_value(&[(42, "foo")]));

        // Limit to no entry
        let result = act_on_range(&array, Range::new(41, 42), Some(0));
        assert_is_empty!(result);

        // Limit to a single entry
        let result = act_on_range(&array, Range::new(41, 42), Some(1));
        assert_eq!(result, u32s_with_vals_to_vec_value(&[(42, "foo")]));
    }

    #[test]
    fn act_on_range_multiple_positions() {
        let mut array = Array::new();
        array.set(39, vkstr("foo"));
        array.set(40, vkstr("bar"));
        array.set(42, vkstr("baz"));
        array.set(45, vkstr("quux"));
        array.set(46, vkstr("quuux"));

        // No limit
        let result = act_on_range(&array, Range::new(40, 45), None);
        assert_eq!(
            result,
            u32s_with_vals_to_vec_value(&[(40, "bar"), (42, "baz"), (45, "quux")])
        );

        // Limit to no entry
        let result = act_on_range(&array, Range::new(40, 45), Some(0));
        assert_is_empty!(result);

        // Limit to a single entry
        let result = act_on_range(&array, Range::new(40, 45), Some(2));
        assert_eq!(
            result,
            u32s_with_vals_to_vec_value(&[(40, "bar"), (42, "baz")])
        );
    }
}
