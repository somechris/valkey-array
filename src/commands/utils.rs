//! Shared utilities for commands

use valkey_module::{ValkeyError, ValkeyResult};

/// Fails with [`WrongArity`](ValkeyError::WrongArity) if the iterator has more entries
pub fn err_if_further_arguments(mut iter: impl Iterator) -> ValkeyResult<()> {
    if iter.next().is_some() {
        return Err(ValkeyError::WrongArity);
    }
    Ok(())
}
