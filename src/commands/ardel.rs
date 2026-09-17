//! Implementation of the `ARDEL` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARDEL` command
pub fn ardel(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let position = arg_iter.next_u64()?;

    err_if_further_arguments(arg_iter)?;

    let count = read_write_action!(ctx, key_name, ValkeyValue::Integer(0), Array::del, position);

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::ardel;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDEL", "foo"]);

        let result = ardel(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDEL", "foo", "23", "bar"]);

        let result = ardel(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDEL", "foo", "bar"]);

        let result = ardel(&ctx, args);
        let err = result.expect_err("ARDEL should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
