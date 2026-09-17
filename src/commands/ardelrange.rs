//! Implementation of the `ARDELRANGE` command

use crate::Array;
use crate::array::Range;
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_write_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(array: &mut Array, (start, end): Range) -> u64 {
    let mut count = 0;

    for position in start..=end {
        count += array.del(&position);
    }
    count
}

/// Implements the `ARDELRANGE` command
pub fn ardelrange(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let range = arg_iter.next_start_end()?;

    err_if_further_arguments(arg_iter)?;

    let count = read_write_action!(ctx, key_name, ValkeyValue::Integer(0), act_on_range, range,);

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::ardelrange;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23"]);

        let result = ardelrange(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23", "42", "bar"]);

        let result = ardelrange(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "bar", "42"]);

        let result = ardelrange(&ctx, args);
        let err = result.expect_err("ARDELRANGE should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARDELRANGE", "foo", "23", "bar"]);

        let result = ardelrange(&ctx, args);
        let err = result.expect_err("ARDELRANGE should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
