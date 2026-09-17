//! Shared utilities for commands

use valkey_module::{ValkeyError, ValkeyResult, ValkeyString};

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

/// Extra utilities for parsing the next arguments
pub trait NextArgExtras {
    /// Parses a single range part
    ///
    /// If the next part is `-` it gets translated to 0.
    /// If the next part is `+` it gets translated to the maximum value.
    /// Otherwise, the next part gets parsed as unsigned integer.
    fn next_start_end_part(&mut self) -> ValkeyResult<u64>;

    /// Parses a start/end pair of arguments
    ///
    /// See [`Self::next_start_end_part`] for the available abbreviations.
    ///
    /// The returned pair is guaranteed that the start is not after the end.
    fn next_start_end(&mut self) -> ValkeyResult<(u64, u64)>;
}

impl<T> NextArgExtras for T
where
    T: Iterator<Item = ValkeyString>,
{
    fn next_start_end_part(&mut self) -> ValkeyResult<u64> {
        self.next()
            .map_or(Err(ValkeyError::WrongArity), |v| match &*v {
                b"-" => Ok(0),
                b"+" => Ok(u64::MAX),
                _ => v.parse_unsigned_integer(),
            })
    }

    fn next_start_end(&mut self) -> ValkeyResult<(u64, u64)> {
        let mut start = self.next_start_end_part()?;
        let mut end = self.next_start_end_part()?;
        if end < start {
            std::mem::swap(&mut end, &mut start);
        }

        Ok((start, end))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::utils::NextArgExtras;
    use crate::test_utils::vkstr;
    use assertables::{assert_contains, assert_matches};
    use valkey_module::{ValkeyError, ValkeyString};

    #[test]
    pub fn next_start_end_part() {
        let parsed = vec![vkstr("-")].into_iter().next_start_end_part().unwrap();
        assert_eq!(parsed, 0);

        let parsed = vec![vkstr("+")].into_iter().next_start_end_part().unwrap();
        assert_eq!(parsed, u64::MAX);

        let parsed = vec![vkstr("42")].into_iter().next_start_end_part().unwrap();
        assert_eq!(parsed, 42);

        let err = Vec::<ValkeyString>::new()
            .into_iter()
            .next_start_end_part()
            .unwrap_err();
        assert_matches!(err, ValkeyError::WrongArity);

        let err = vec![vkstr("-1")]
            .into_iter()
            .next_start_end_part()
            .unwrap_err();
        assert_contains!(err.to_string(), "parse");
    }

    #[test]
    pub fn next_start_end() {
        // Standard range
        let parsed = vec![vkstr("23"), vkstr("42")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (23, 42));

        // Standard range with max
        let parsed = vec![vkstr("23"), vkstr("+")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (23, u64::MAX));

        // Standard range with min
        let parsed = vec![vkstr("-"), vkstr("42")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (0, 42));

        // min / max
        let parsed = vec![vkstr("-"), vkstr("+")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (0, u64::MAX));

        // Standard range reversed
        let parsed = vec![vkstr("42"), vkstr("23")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (23, 42));

        // min / max
        let parsed = vec![vkstr("+"), vkstr("-")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (0, u64::MAX));

        // min at end
        let parsed = vec![vkstr("23"), vkstr("-")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (0, 23));

        // max at start
        let parsed = vec![vkstr("+"), vkstr("42")]
            .into_iter()
            .next_start_end()
            .unwrap();
        assert_eq!(parsed, (42, u64::MAX));

        let err = Vec::<ValkeyString>::new()
            .into_iter()
            .next_start_end()
            .unwrap_err();
        assert_matches!(err, ValkeyError::WrongArity);

        let err = vec![vkstr("42")].into_iter().next_start_end().unwrap_err();
        assert_matches!(err, ValkeyError::WrongArity);

        let err = vec![vkstr("-1")].into_iter().next_start_end().unwrap_err();
        assert_contains!(err.to_string(), "parse");

        let err = vec![vkstr("42"), vkstr("-1")]
            .into_iter()
            .next_start_end()
            .unwrap_err();
        assert_contains!(err.to_string(), "parse");
    }
}
