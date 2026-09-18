//! Implementation of the `ARGET` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{err_if_further_arguments, read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

fn retrieve(array: &Array, position: u64) -> Option<ValkeyString> {
    array.get(position).cloned()
}

/// Implements the `ARGET` command
pub fn arget(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let position = arg_iter.next_u64()?;

    err_if_further_arguments(arg_iter)?;

    let maybe_element = read_only_action!(ctx, key_name, ValkeyValue::Null, retrieve, position);

    Ok(maybe_element.into())
}

#[cfg(test)]
mod tests {
    use crate::commands::arget;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGET", "foo"]);

        let result = arget(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGET", "foo", "23", "bar"]);

        let result = arget(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGET", "foo", "bar"]);

        let result = arget(&ctx, args);
        let err = result.expect_err("ARGET should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
