//! Implementation of the `ARGREP` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(
    array: &mut Array,
    mut start: u64,
    mut end: u64,
    search_expr: &ValkeyString,
) -> Vec<ValkeyValue> {
    if end < start {
        std::mem::swap(&mut end, &mut start);
    }

    let mut ret = vec![];
    for position in start..=end {
        if let Some(value) = array.get(&position)
            && value == *search_expr
        {
            ret.push((position as i64).into());
        }
    }
    ret
}

/// Implements the `ARGREP` command
pub fn argrep(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let start = arg_iter.next_u64()?;
    let end = arg_iter.next_u64()?;

    if arg_iter
        .next_arg()?
        .to_string()
        .to_ascii_uppercase()
        .as_str()
        != "EXACT"
    {
        return Err(ValkeyError::Str("ERR Unknown ARGREP operation"));
    }

    let search_expr = &arg_iter.next_arg()?;

    err_if_further_arguments(arg_iter)?;

    let items = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        start,
        end,
        search_expr,
    );

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::commands::argrep;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "42"]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "42", "EXACT", "bar", "baz"]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "bar", "42"]);

        let result = argrep(&ctx, args);
        let err = result.expect_err("ARGREP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "bar"]);

        let result = argrep(&ctx, args);
        let err = result.expect_err("ARGREP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_unsupported_operation() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "42", "bar"]);

        let result = argrep(&ctx, args);
        let err = result.expect_err("ARGREP should fail");

        assert_contains!(err.to_string(), "operation");
    }
}
