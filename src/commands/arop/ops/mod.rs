//! Operations for `AROP`

use valkey_module::{ValkeyString, ValkeyValue};

mod sum;
mod used;

pub use sum::SumOperation;
pub use used::UsedOperation;

/// An [`Operation`]s functions
pub trait Operation {
    fn new() -> Self;

    /// Accumulates a found value
    fn accumulate(&mut self, value: ValkeyString);

    /// Builds an accumulator's result
    fn build_result(self) -> ValkeyValue;
}
