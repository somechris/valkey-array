//! Implementation of the `ARGETRANGE` command

use crate::Array;
use crate::array::{ArrayType, Range};
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_only_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(array: &Array, range: Range) -> Vec<ValkeyValue> {
    array
        .range_iter(range)
        .map(|(_position, maybe_value)| match maybe_value {
            Some(str) => str.into(),
            None => ValkeyValue::Null,
        })
        .collect()
}

/// Implements the `ARGETRANGE` command
pub fn argetrange(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let range = arg_iter.next_range()?;

    err_if_further_arguments(arg_iter)?;

    let items = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        range,
    );

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::commands::argetrange;
    use crate::utils::test_utils::{assert_arity_error, assert_position_error};

    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "23"]);

        let result = argetrange(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "23", "42", "bar"]);

        let result = argetrange(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "bar", "42"]);

        let result = argetrange(&ctx, args);
        assert_position_error(result);
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "23", "bar"]);

        let result = argetrange(&ctx, args);
        assert_position_error(result);
    }
}
