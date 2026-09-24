//! Implementation of the `ARCOUNT` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{err_if_further_arguments, read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARCOUNT` command
pub fn arcount(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    err_if_further_arguments(arg_iter)?;

    let count = read_only_action!(ctx, key_name, ValkeyValue::Integer(0), Array::count);

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::arcount;
    use crate::test_utils::assert_arity_error;

    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARCOUNT"]);

        let result = arcount(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARCOUNT", "foo", "bar"]);

        let result = arcount(&ctx, args);

        assert_arity_error(result);
    }
}
