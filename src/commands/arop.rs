//! Implementation of the `AROP` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(array: &mut Array, mut start: u64, mut end: u64) -> ValkeyValue {
    if end < start {
        std::mem::swap(&mut end, &mut start);
    }

    let mut acc: i64 = 0;
    for position in start..=end {
        if array.get(&position).is_some() {
            acc += 1;
        }
    }

    acc.into()
}

/// Implements the `AROP` command
pub fn arop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let start = &arg_iter.next_u64()?;
    let end = &arg_iter.next_u64()?;

    // Parse operation
    if arg_iter.next_arg()?.to_string().to_ascii_uppercase().as_str() != "USED" {
        return Err(ValkeyError::Str("ERR Unknown AROP operation"))
    }

    err_if_further_arguments(arg_iter)?;

    let result = read_write_action!(ctx, key_name, ValkeyValue::Array(Vec::new()), act_on_range, *start, *end);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::commands::arop;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "42"]);

        let result = arop(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "42", "USED", "bar"]);

        let result = arop(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "bar", "42", "USED"]);

        let result = arop(&ctx, args);
        let err = result.expect_err("AROP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "bar", "USED"]);

        let result = arop(&ctx, args);
        let err = result.expect_err("AROP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_unknown_operation() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "24", "BAZ"]);

        let result = arop(&ctx, args);
        let err = result.expect_err("AROP should fail");

        assert_contains!(err.to_string(), "operation");
    }
}
