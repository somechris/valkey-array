//! Implementation of the `ARINFO` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{err_if_further_arguments, read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use std::collections::HashMap;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARINFO` command
pub fn arinfo(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let mut full = false;

    if let Some(option) = arg_iter.next() {
        if option.to_string().to_ascii_uppercase().as_str() == "FULL" {
            full = true;
        } else {
            return Err(ValkeyError::WrongArity);
        }

        err_if_further_arguments(arg_iter)?;
    }

    let map = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Map(HashMap::new()),
        Array::info,
        full
    );

    Ok(map.into())
}

#[cfg(test)]
mod tests {
    use crate::commands::arinfo;
    use crate::test_utils::assert_arity_error;

    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINFO"]);

        let result = arinfo(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high_without_full() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINFO", "foo", "bar"]);

        let result = arinfo(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high_with_full() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINFO", "foo", "FULL", "bar"]);

        let result = arinfo(&ctx, args);

        assert_arity_error(result);
    }
}
