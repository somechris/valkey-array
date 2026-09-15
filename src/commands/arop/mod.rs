//! Implementation of the `AROP` command

mod ops;

use crate::Array;
use crate::commands::arop::ops::Operation;
use crate::commands::utils::{err_if_further_arguments, read_write_action, to_arg_iter};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Substitutes for available operations
///
/// These are needed to parse the operation (and eventually failing) before opening the key.
pub enum SubstituteOperation {
    /// Converts numeric items to integers and binary `AND`s them (Null if there were no items)
    And,
    /// Maximum of numeric items (Null if there were no items)
    Max,
    /// Minimum of numeric items (Null if there were no items)
    Min,
    /// Converts numeric items to integers and binary `OR`s them (Null if there were no items)
    Or,
    /// Sums numeric items
    Sum,
    /// Counts the used positions
    Used,
}

/// Executes the command on each position in the range (inclusive)
fn act_on_range_typed<OP: Operation>(
    array: &mut Array,
    mut start: u64,
    mut end: u64,
    mut op: OP,
) -> ValkeyResult {
    if end < start {
        std::mem::swap(&mut end, &mut start);
    }

    //let (accumulate, build_result) = op.get_funcs();
    for position in start..=end {
        if let Some(value) = array.get(&position) {
            op.accumulate(value);
        }
    }

    Ok(op.build_result())
}

fn act_on_range(
    array: &mut Array,
    start: u64,
    end: u64,
    op_subst: SubstituteOperation,
) -> ValkeyResult {
    use SubstituteOperation::*;
    match op_subst {
        And => act_on_range_typed(array, start, end, ops::AndOperation::new()),
        Max => act_on_range_typed(array, start, end, ops::MaxOperation::new()),
        Min => act_on_range_typed(array, start, end, ops::MinOperation::new()),
        Or => act_on_range_typed(array, start, end, ops::OrOperation::new()),
        Sum => act_on_range_typed(array, start, end, ops::SumOperation::new()),
        Used => act_on_range_typed(array, start, end, ops::UsedOperation::new()),
    }
}

/// Implements the `AROP` command
pub fn arop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let start = &arg_iter.next_u64()?;
    let end = &arg_iter.next_u64()?;

    // Parse operation
    let op_subst = match arg_iter
        .next_arg()?
        .to_string()
        .to_ascii_uppercase()
        .as_str()
    {
        "AND" => SubstituteOperation::And,
        "MAX" => SubstituteOperation::Max,
        "MIN" => SubstituteOperation::Min,
        "OR" => SubstituteOperation::Or,
        "SUM" => SubstituteOperation::Sum,
        "USED" => SubstituteOperation::Used,
        _ => return Err(ValkeyError::Str("ERR Unknown AROP operation")),
    };

    err_if_further_arguments(arg_iter)?;

    read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        *start,
        *end,
        op_subst
    )
}

#[cfg(test)]
mod tests {
    use crate::commands::arop;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "42"]);

        let result = arop(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "42", "USED", "bar"]);

        let result = arop(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "bar", "42", "USED"]);

        let result = arop(&ctx, args);
        let err = result.expect_err("AROP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "bar", "USED"]);

        let result = arop(&ctx, args);
        let err = result.expect_err("AROP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_unknown_operation() {
        let ctx = Context::test();
        let args = create_test_args(&["AROP", "foo", "23", "24", "BAZ"]);

        let result = arop(&ctx, args);
        let err = result.expect_err("AROP should fail");

        assert_contains!(err.to_string(), "operation");
    }
}
