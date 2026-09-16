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
    opt_limit: Option<u64>,
    case_sensitive: bool,
) -> Vec<ValkeyValue> {
    if end < start {
        std::mem::swap(&mut end, &mut start);
    }

    let (limited, limit) = match opt_limit {
        Some(limit) => (true, limit as usize),
        None => (false, 0),
    };

    let comp_fn: &dyn Fn(ValkeyString) -> bool = if case_sensitive {
        &|candidate| candidate == *search_expr
    } else {
        &|candidate: ValkeyString| candidate.eq_ignore_ascii_case(search_expr)
    };

    let mut ret = vec![];
    for position in start..=end {
        if let Some(value) = array.get(&position)
            && comp_fn(value)
        {
            ret.push((position as i64).into());
        }

        // Checking an eventual limit
        if limited && ret.len() == limit {
            break;
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
    let mut opt_limit = None;
    let mut case_sensitive = true;

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

    // Parse optional limit
    while let Some(arg) = arg_iter.next() {
        match arg.to_string().to_ascii_uppercase().as_str() {
            "LIMIT" => opt_limit = Some(arg_iter.next_u64()?),
            "NOCASE" => case_sensitive = false,
            _ => return Err(ValkeyError::WrongArity),
        }
    }

    err_if_further_arguments(arg_iter)?;

    let items = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        start,
        end,
        search_expr,
        opt_limit,
        case_sensitive,
    );

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::Array;
    use crate::commands::argrep;
    use crate::commands::argrep::act_on_range;
    use crate::test_utils::{u32s_to_vec_value, vkstr};
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
    fn arity_too_high_with_limit() {
        let ctx = Context::test();
        let args = create_test_args(&[
            "ARGREP", "foo", "23", "42", "EXACT", "bar", "LIMIT", "4711", "baz",
        ]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_no_case() {
        let ctx = Context::test();
        let args =
            create_test_args(&["ARGREP", "foo", "23", "42", "EXACT", "bar", "NOCASE", "baz"]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_all_options() {
        let ctx = Context::test();
        let args = create_test_args(&[
            "ARGREP", "foo", "23", "42", "EXACT", "bar", "LIMIT", "4711", "NOCASE", "baz",
        ]);

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

    #[test]
    fn act_on_range_exact_case_sensitive() {
        let mut array = Array::new();
        array.set(&0, &vkstr("foo")); // ignored (not in range)
        array.set(&1, &vkstr("foo")); // match
        // No position 2, no match
        array.set(&3, &vkstr("foo")); // match
        array.set(&4, &vkstr("fOo")); // no match (we're case-sensitive)
        array.set(&5, &vkstr("foo")); // match
        array.set(&6, &vkstr("foo")); // ignored (not in range)

        let result = act_on_range(&mut array, 1, 5, &vkstr("foo"), None, true);
        let expected = u32s_to_vec_value(&[1, 3, 5]);
        assert_eq!(result, expected);
    }

    #[test]
    fn act_on_range_exact_case_insensitive() {
        let mut array = Array::new();
        array.set(&0, &vkstr("foo")); // ignored (not in range)
        array.set(&1, &vkstr("foo")); // match
        // No position 2, no match
        array.set(&3, &vkstr("foo")); // match
        array.set(&4, &vkstr("fOo")); // match (we're case-insensitive)
        array.set(&5, &vkstr("FOO")); // match (we're case-insensitive)
        array.set(&6, &vkstr("foo")); // ignored (not in range)

        let result = act_on_range(&mut array, 1, 5, &vkstr("foo"), None, false);
        let expected = u32s_to_vec_value(&[1, 3, 4, 5]);
        assert_eq!(result, expected);
    }

    #[test]
    fn act_on_range_limit() {
        let mut array = Array::new();
        array.set(&0, &vkstr("foo")); // ignored (not in range)
        array.set(&1, &vkstr("foo")); // match
        // No position 2, no match
        array.set(&3, &vkstr("foo")); // match
        array.set(&4, &vkstr("bar")); // no match (different text)
        array.set(&5, &vkstr("foo")); // match
        array.set(&6, &vkstr("foo")); // ignored (not in range)

        let result = act_on_range(&mut array, 1, 5, &vkstr("foo"), Some(2), true);
        let expected = u32s_to_vec_value(&[1, 3]);
        assert_eq!(result, expected);
    }
}
