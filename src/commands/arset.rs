//! Implementation of the `ARSET` command

use crate::commands::utils::err_if_further_arguments;
use crate::types::{ARRAY_TYPE, Array};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARSET` command
pub fn arset(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;
    let position = &args.next_u64()?;
    let value = &args.next_arg()?;

    err_if_further_arguments(args)?;

    ctx.log_warning(&format!(
        "Running ARSET for {key_name} @ {position} = {value}"
    ));

    let key = ctx.open_key_writable(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&ARRAY_TYPE) else {
        return Err(ValkeyError::WrongType);
    };

    let new_slot_count = match maybe_array {
        Some(array) => array.set(position, value),
        None => {
            let mut array = Array::new();
            let ret = array.set(position, value);
            if key.set_value(&ARRAY_TYPE, array).is_err() {
                return Err(ValkeyError::Str("Failed to set value"));
            }
            ret
        }
    };

    Ok(ValkeyValue::Integer(new_slot_count as i64))
}
