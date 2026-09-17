//! Implementation of the `ARINSERT` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_creating_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARINSERT` command
pub fn arinsert(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let value = arg_iter.next_arg()?;

    err_if_further_arguments(arg_iter)?;

    let count = read_write_creating_action!(ctx, key_name, Array::insert, value);

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::arinsert;
    use assertables::assert_matches;
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINSERT", "foo"]);

        let result = arinsert(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINSERT", "foo", "bar", "baz"]);

        let result = arinsert(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }
}
