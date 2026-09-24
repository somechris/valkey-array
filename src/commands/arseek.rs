//! Implementation of the `ARSEEK` command

use super::utils::{NextArgExtras, read_write_action, to_arg_iter};
use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::err_if_further_arguments;
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARSEEK` command
pub fn arseek(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let position = &arg_iter.next_position()?;

    err_if_further_arguments(arg_iter)?;

    let position = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Integer(0),
        Array::set_insert_cursor,
        *position
    );

    Ok(ValkeyValue::Integer(position as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::arseek;
    use crate::test_utils::{assert_arity_error, assert_position_error};

    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSEEK", "foo"]);

        let result = arseek(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSEEK", "foo", "23", "bar"]);

        let result = arseek(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSEEK", "foo", "bar"]);

        let result = arseek(&ctx, args);
        assert_position_error(result);
    }
}
