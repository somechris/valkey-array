//! Implementation of the `ARLEN` command

use crate::Array;
use crate::commands::utils::err_if_further_arguments;
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARLEN` command
pub fn arlen(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;

    err_if_further_arguments(args)?;

    let key = ctx.open_key(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
        return Err(ValkeyError::WrongType);
    };

    let count = match maybe_array {
        Some(array) => array.next_highest_position(),
        None => 0,
    };

    Ok(ValkeyValue::Integer(count as i64))
}
