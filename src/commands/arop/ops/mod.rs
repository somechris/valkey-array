//! Operations for `AROP`

use valkey_module::{ValkeyString, ValkeyValue};

mod and;
mod r#match;
mod max;
mod min;
mod or;
mod sum;
mod used;
mod xor;

pub mod utils;

pub use and::AndOperation;
pub use r#match::MatchOperation;
pub use max::MaxOperation;
pub use min::MinOperation;
pub use or::OrOperation;
pub use sum::SumOperation;
pub use used::UsedOperation;
pub use xor::XorOperation;

/// An [`Operation`]s functions
pub trait Operation {
    /// Accumulates a found value
    fn accumulate(&mut self, value: ValkeyString);

    /// Builds an accumulator's result
    fn build_result(self) -> ValkeyValue;
}
