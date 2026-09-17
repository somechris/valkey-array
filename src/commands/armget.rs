//! Implementation of the `ARMGET` command

use crate::Array;
use crate::commands::utils::{read_only_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on index of the iterator
fn act_on_indices(
    array: &Array,
    iter: impl Iterator<Item = ValkeyString>,
) -> ValkeyResult<Vec<ValkeyValue>> {
    iter.map(|position_str| Ok(array.get(position_str.parse_unsigned_integer()?).into()))
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
    use assertables::assert_matches;
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARMGET"]);

        let result = armget(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }
}
