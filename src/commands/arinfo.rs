//! Implementation of the `ARINFO` command

use crate::Array;
use crate::commands::utils::{err_if_further_arguments, read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use std::collections::HashMap;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARINFO` command
pub fn arinfo(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    err_if_further_arguments(arg_iter)?;

    let map = read_only_action!(ctx, key_name, ValkeyValue::Map(HashMap::new()), Array::info);

    Ok(map.into())
}

#[cfg(test)]
mod tests {
    use crate::commands::arinfo;
    use assertables::assert_matches;
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINFO"]);

        let result = arinfo(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARINFO", "foo", "bar"]);

        let result = arinfo(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }
}
