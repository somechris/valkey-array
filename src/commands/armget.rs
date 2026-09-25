//! Implementation of the `ARMGET` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{ValkeyStringExtras, read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on index of the iterator
fn act_on_indices(array: &Array, positions: Vec<u64>) -> ValkeyResult<Vec<ValkeyValue>> {
    positions
        .into_iter()
        .map(|position| Ok(array.get(position).into()))
        .collect()
}

/// Implements the `ARMGET` command
pub fn armget(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let positions = arg_iter
        .map(|s| s.parse_position())
        .collect::<ValkeyResult<Vec<u64>>>()?;

    let items = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_indices,
        positions
    )?;

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::commands::armget;
    use crate::utils::test_utils::assert_arity_error;

    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARMGET"]);

        let result = armget(&ctx, args);

        assert_arity_error(result);
    }
}
