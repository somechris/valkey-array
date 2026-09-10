//! Implementation of the `ARSET` command

use crate::Array;
use crate::commands::utils::err_if_further_arguments;
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARSET` command
pub fn arset(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;
    let position = &args.next_u64()?;
    let value = &args.next_arg()?;

    err_if_further_arguments(args)?;

    let key = ctx.open_key_writable(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
        return Err(ValkeyError::WrongType);
    };

    let new_slot_count = match maybe_array {
        Some(array) => array.set(position, value),
        None => {
            let mut array = Array::new();
            let ret = array.set(position, value);
            if key.set_value(&VKARRAY, array).is_err() {
                return Err(ValkeyError::Str("Failed to set value"));
            }
            ret
        }
    };

    Ok(ValkeyValue::Integer(new_slot_count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::arset;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "23"]);

        let result = arset(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "23", "bar", "baz"]);

        let result = arset(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type() {
        let ctx = Context::test();
        let args = create_test_args(&["ARSET", "foo", "bar", "baz"]);

        let result = arset(&ctx, args);
        let err = result.expect_err("ARSET should fail");

        assert_contains!(err.to_string(), "integer");
    }
}
