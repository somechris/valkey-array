//! Implementation of the `ARSET` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_creating_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARSET` command
pub fn arset(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let position = arg_iter.next_u64()?;
    let value = arg_iter.next_arg()?;

    err_if_further_arguments(arg_iter)?;

    let count = read_write_creating_action!(ctx, key_name, Array::set, position, value);

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::arset;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "23"]);

        let result = arset(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "23", "bar", "baz"]);

        let result = arset(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "bar", "baz"]);

        let result = arset(&ctx, args);
        let err = result.expect_err("ARSET should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
