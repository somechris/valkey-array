//! The Rust implementation of Valkey arrays
//!
//! This gets registered in Valkey via [`crate::types::VKARRAY`].

use std::collections::HashMap;
use valkey_module::ValkeyString;

/// The struct that models the data in Rust
#[derive(Default, Debug)]
pub struct Array {
    values: HashMap<u64, ValkeyString>,
    next_highest_position: u64,
}

impl Array {
    /// Builds a new instance
    pub fn new() -> Self {
        Array::default()
    }

    /// Gets the number of entries
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// Gets the highest used position + 1
    ///
    /// If there are no entries in the array, `0` is returned
    pub fn next_highest_position(&self) -> u64 {
        self.next_highest_position
    }

    /// Deletes the value at a given position
    pub fn del(&mut self, position: &u64) -> Option<ValkeyString> {
        self.values.remove(position)
    }

    /// Gets the value at a given position
    pub fn get(&self, position: &u64) -> Option<&ValkeyString> {
        self.values.get(position)
    }

    /// Sets the value at a given position
    ///
    /// # Returns
    ///
    /// If the slot was previously unused, the function returns `1`. Otherwise `0`.
    pub fn set(&mut self, position: &u64, value: &ValkeyString) -> usize {
        self.next_highest_position = self.next_highest_position.max(position + 1);
        if self.values.insert(*position, value.clone()).is_some() {
            // The position already had a value, so it's not a new slot
            0
        } else {
            // The position previously did not have a value, so it's a new slot
            1
        }
    }
}
