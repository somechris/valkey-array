//! Implementation of the `ARMSET` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{ValkeyStringExtras, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on index/value pairs of the iterator
fn act_on_pairs(array: &mut Array, pairs: Vec<(u64, ValkeyString)>) -> u64 {
    let mut count = 0;
    for (position, value) in pairs {
        count += array.set(position, value);
    }
    count
}

/// Implements the `ARMSET` command
pub fn armset(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    // We build pairs upfront to detect arity issues before touching the array
    let mut pairs: Vec<(u64, ValkeyString)> = Vec::new();
    while let Some(position_str) = arg_iter.next() {
        let position = position_str.parse_position()?;
        let value = arg_iter.next_arg()?;
        pairs.push((position, value));
    }

    let count = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_pairs,
        pairs
    );

    Ok(ValkeyValue::Integer(count as i64))
}

#[cfg(test)]
mod tests {
    use crate::commands::armset;
    use assertables::assert_matches;
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARMSET"]);

        let result = armset(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_position_without_value() {
        let ctx = Context::test();
        let args = create_test_args(&["ARMSET", "foo", "23"]);

        let result = armset(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }
}
