//! Implementation of the `ARSCAN` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on each position in the range (inclusive)
fn act_on_range(array: &mut Array, mut start: u64, mut end: u64, opt_limit: Option<u64>) -> Vec<ValkeyValue> {
    if end < start {
        std::mem::swap(&mut end, &mut start);
    }

    let (limited, limit) = match opt_limit {
        Some(limit) => (true, limit as usize),
        None => (false, 0),
    };

    let mut ret = vec![];
    for position in start..=end {
        if let Some(value) = array.get(&position) {
            ret.push((vec![ValkeyValue::from(position as i64), value.into()]).into());
        }

        // Checking an eventual limit
        if limited && ret.len() == limit {
            break;
        }
    }
    ret
}

/// Implements the `ARSCAN` command
pub fn arscan(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let start = &arg_iter.next_u64()?;
    let end = &arg_iter.next_u64()?;
    let mut opt_limit = None;

    // Parse optional limit
    if let Some(arg) = arg_iter.next() {
        match arg.to_string().to_ascii_uppercase().as_str() {
            "LIMIT" => opt_limit =Some(arg_iter.next_u64()?),
            _ => return Err(ValkeyError::WrongArity),
        }
        err_if_further_arguments(arg_iter)?;
    }

    let items = read_write_action!(ctx, key_name, ValkeyValue::Array(Vec::new()), act_on_range, *start, *end, opt_limit);

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::commands::arscan;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23"]);

        let result = arscan(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_without_limit() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "42", "bar"]);

        let result = arscan(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_limit() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "42", "LIMIT", "4711", "bar"]);

        let result = arscan(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "bar", "42"]);

        let result = arscan(&ctx, args);
        let err = result.expect_err("ARSCAN should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "bar"]);

        let result = arscan(&ctx, args);
        let err = result.expect_err("ARSCAN should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_limit() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSCAN", "foo", "23", "42", "LIMIT", "bar"]);

        let result = arscan(&ctx, args);
        let err = result.expect_err("ARSCAN should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
