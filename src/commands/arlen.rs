//! Implementation of the `ARLEN` command

use super::utils::{read_only_action, to_arg_iter};
use crate::Array;
use crate::commands::utils::err_if_further_arguments;
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARLEN` command
pub fn arlen(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    err_if_further_arguments(arg_iter)?;

    let nhp = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Integer(0),
        Array::next_highest_position
    );

    Ok(ValkeyValue::Integer(nhp as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::arlen;
    use assertables::assert_matches;
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARLEN"]);

        let result = arlen(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARLEN", "foo", "bar"]);

        let result = arlen(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }
}
