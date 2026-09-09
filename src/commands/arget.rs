//! Implementation of the `ARGET` command

use crate::commands::utils::err_if_further_arguments;
use crate::types::Array;
use crate::types::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARGET` command
pub fn arget(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;
    let position = &args.next_u64()?;

    err_if_further_arguments(args)?;

    let key = ctx.open_key(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
        return Err(ValkeyError::WrongType);
    };

    let Some(array) = maybe_array else {
        return Ok(ValkeyValue::Null);
    };

    let value = match array.get(position) {
        Some(ref_value) => ValkeyValue::BulkValkeyString(ref_value.clone()),
        None => ValkeyValue::Null,
    };

    Ok(value)
}
