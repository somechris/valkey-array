//! Implementation of the `ARGETRANGE` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(array: &mut Array, mut start: u64, mut end: u64) -> Vec<ValkeyValue> {
    if end < start {
        std::mem::swap(&mut end, &mut start);
    }

    (start..=end)
        .map(|position| match array.get(&position) {
            Some(str) => str.into(),
            None => ValkeyValue::Null,
        })
        .collect()
}

/// Implements the `ARGETRANGE` command
pub fn argetrange(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let start = &arg_iter.next_u64()?;
    let end = &arg_iter.next_u64()?;

    err_if_further_arguments(arg_iter)?;

    let items = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        *start,
        *end
    );

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::commands::argetrange;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "23"]);

        let result = argetrange(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "23", "42", "bar"]);

        let result = argetrange(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "bar", "42"]);

        let result = argetrange(&ctx, args);
        let err = result.expect_err("ARGETRANGE should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGETRANGE", "foo", "23", "bar"]);

        let result = argetrange(&ctx, args);
        let err = result.expect_err("ARGETRANGE should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
