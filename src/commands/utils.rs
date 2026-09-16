//! Shared utilities for commands

use valkey_module::{ValkeyError, ValkeyResult};

/// Converts a command's `args` into an iterator over the relevant arguments
macro_rules! to_arg_iter {
    ($args:expr) => {
        $args.into_iter().skip(1)
    };
}
pub(crate) use to_arg_iter;

/// Fails with [`WrongArity`](ValkeyError::WrongArity) if the iterator has more entries
pub fn err_if_further_arguments(mut iter: impl Iterator) -> ValkeyResult<()> {
    if iter.next().is_some() {
        return Err(ValkeyError::WrongArity);
    }
    Ok(())
}

/// Runs a read-only action on an array
///
/// # Arguments
///
/// * `$ctx` - Context to operate on
/// * `$key_name` - Name of the key to operate on
/// * `$default` - Value to return if the `$key_name` does not exist
/// * `$func` - The function to run on the `$key_name`'s array
/// * `$arg` - All remaining arguments get passed to $func
///
/// # Errors
///
/// If the key exists and is not an array, a `WRONGTYPE` error is returned
macro_rules! read_only_action {
    ($ctx:expr, $key_name:expr, $default:expr, $func:expr $(,$arg:expr)*) => {{
        let key = $ctx.open_key($key_name);
        let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
            return Err(ValkeyError::WrongType);
        };

        let Some(array) = maybe_array else {
            return Ok($default);
        };

        $func(array$(, $arg)*)
    }}
}
pub(crate) use read_only_action;

/// Runs a read/write action on an array, but does not create the key, if it does not exist
///
/// # Arguments
///
/// * `$ctx` - Context to operate on
/// * `$key_name` - Name of the key to operate on
/// * `$default` - Value to return if the `$key_name` does not exist
/// * `$func` - The function to run on the `$key_name`'s array
/// * `$arg` - All remaining arguments get passed to $func
///
/// # Errors
///
/// If the key exists and is not an array, a `WRONGTYPE` error is returned
macro_rules! read_write_action {
    ($ctx:expr, $key_name:expr, $default:expr, $func:expr $(,$arg:expr)* $(,)?) => {{
        let key = $ctx.open_key_writable($key_name);
        let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
            return Err(ValkeyError::WrongType);
        };

        let Some(array) = maybe_array else {
            return Ok($default);
        };

        $func(array$(, $arg)*)
    }}
}
pub(crate) use read_write_action;

/// Runs a read/write action on an array, creating the key, if it does not exist
///
/// # Arguments
///
/// * `$ctx` - Context to operate on
/// * `$key_name` - Name of the key to operate on
/// * `$func` - The function to run on the `$key_name`'s array
/// * `$arg` - All remaining arguments get passed to $func
///
/// # Errors
///
/// If the key exists and is not an array, a `WRONGTYPE` error is returned
macro_rules! read_write_creating_action {
    ($ctx:expr, $key_name:expr, $func:expr $(,$arg:expr)* $(,)?) => {{
        let key = $ctx.open_key_writable($key_name);
        let Ok(maybe_array) = key.get_value::<Array>(&VKARRAY) else {
            return Err(ValkeyError::WrongType);
        };

        match maybe_array {
            Some(array) => $func(array$(, $arg)*),
            None => {
                let mut array = Array::new();
                let ret = $func(&mut array$(, $arg)*);
                if key.set_value(&VKARRAY, array).is_err() {
                    return Err(ValkeyError::Str("Failed to set value"));
                }
                ret
            }
        }
    }}
}
pub(crate) use read_write_creating_action;
