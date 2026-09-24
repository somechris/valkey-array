//! Implementation of the `ARMGET` command

use crate::Array;
use crate::array::ArrayType;
use crate::commands::utils::{ValkeyStringExtras, read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on index of the iterator
fn act_on_indices(
    array: &Array,
    iter: impl Iterator<Item = ValkeyString>,
) -> ValkeyResult<Vec<ValkeyValue>> {
    iter.map(|position_str| Ok(array.get(position_str.parse_position()?).into()))
        .collect()
}

/// Implements the `ARMGET` command
pub fn armget(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;

    let items = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_indices,
        arg_iter
    )?;

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    use crate::commands::armget;
    use crate::test_utils::assert_arity_error;

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
