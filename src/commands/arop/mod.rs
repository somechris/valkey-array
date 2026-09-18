//! Implementation of the `AROP` command

mod ops;

use crate::Array;
use crate::array::{ArrayType, Range};
use crate::commands::arop::ops::Operation;
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_write_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Substitutes for available operations
///
/// These are needed to parse the operation (and eventually failing) before opening the key.
pub enum SubstituteOperation {
    /// Converts numeric items to integers and binary `AND`s them (Null if there were no items)
    And,
    /// Counts the number of items that exactly match the search expression
    Match(ValkeyString),
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
    /// Converts numeric items to integers and binary `XOR`s them (Null if there were no items)
    Xor,
}

/// Executes the command on each position in the range (inclusive)
fn act_on_range_typed<OP: Operation>(array: &mut Array, range: Range, mut op: OP) -> ValkeyResult {
    for (_position, maybe_value) in array.range_iter(range) {
        if let Some(value) = maybe_value {
            op.accumulate(value);
        }
    }

    Ok(op.build_result())
}

fn act_on_range(array: &mut Array, range: Range, op_subst: SubstituteOperation) -> ValkeyResult {
    use SubstituteOperation::*;
    match op_subst {
        And => act_on_range_typed(array, range, ops::AndOperation::new()),
        Max => act_on_range_typed(array, range, ops::MaxOperation::new()),
        Match(search_expr) => {
            act_on_range_typed(array, range, ops::MatchOperation::new(search_expr))
        }
        Min => act_on_range_typed(array, range, ops::MinOperation::new()),
        Or => act_on_range_typed(array, range, ops::OrOperation::new()),
        Sum => act_on_range_typed(array, range, ops::SumOperation::new()),
        Used => act_on_range_typed(array, range, ops::UsedOperation::new()),
        Xor => act_on_range_typed(array, range, ops::XorOperation::new()),
    }
}

/// Implements the `AROP` command
pub fn arop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let range = arg_iter.next_start_end()?;

    // Parse operation
    let op_subst = match arg_iter
        .next_arg()?
        .to_string()
        .to_ascii_uppercase()
        .as_str()
    {
        "AND" => SubstituteOperation::And,
        "MATCH" => {
            let search_expr = arg_iter.next_arg()?;
            SubstituteOperation::Match(search_expr)
        }
        "MAX" => SubstituteOperation::Max,
        "MIN" => SubstituteOperation::Min,
        "OR" => SubstituteOperation::Or,
        "SUM" => SubstituteOperation::Sum,
        "USED" => SubstituteOperation::Used,
        "XOR" => SubstituteOperation::Xor,
        _ => return Err(ValkeyError::Str("ERR Unknown AROP operation")),
    };

    err_if_further_arguments(arg_iter)?;

    read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        range,
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
