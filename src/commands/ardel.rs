//! Implementation of the `ARDEL` command

use crate::Array;
use crate::commands::utils::err_if_further_arguments;
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Implements the `ARDEL` command
pub fn ardel(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;
    let position = &args.next_u64()?;

    err_if_further_arguments(args)?;

    let key = ctx.open_key_writable(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
        return Err(ValkeyError::WrongType);
    };

    let maybe_deleted = match maybe_array {
        Some(array) => array.del(position),
        None => {
            let mut array = Array::new();
            let ret = array.del(position);
            if key.set_value(&VKARRAY, array).is_err() {
                return Err(ValkeyError::Str("Failed to set value"));
            }
            ret
        }
    };

    let deleted_count = match maybe_deleted {
        Some(_) => 1,
        None => 0,
    };

    Ok(ValkeyValue::Integer(deleted_count))
}
