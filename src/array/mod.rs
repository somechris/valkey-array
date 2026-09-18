//! The Rust implementation of Valkey arrays
//!
//! This gets registered in Valkey via [`crate::registration::VKARRAY`].

mod array_range_iter;
mod single_hash_map_array;

use crate::array::array_range_iter::GenericArrayRangeIter;
use std::collections::HashMap;
use valkey_module::ValkeyString;

/// Type for position ranges
pub type Range = (u64, u64);

/// The actual type to hold the data stored in Valkey
pub type Array = single_hash_map_array::SingleHashMapArray;

/// The required functions to be used as array implementation for `valkey-array`
pub trait ArrayType: Sized {
    /// Builds a new instance
    fn new() -> Self;

    /// Gets the number of entries
    fn count(&self) -> usize;

    /// Gets the highest used position + 1
    ///
    /// If there are no entries in the array, `0` is returned
    fn next_highest_position(&self) -> u64;

    /// Deletes the value at a given position
    fn del(&mut self, position: u64) -> u64;

    /// Gets the value at a given position
    fn get(&self, position: u64) -> Option<&ValkeyString>;

    /// Gets the position where the next item will get inserted
    fn get_insert_cursor(&self) -> u64;

    /// Gets the value at a given position
    fn insert(&mut self, value: ValkeyString) -> u64;

    /// Inserts an element in ring-buffer fashion
    ///
    /// # Returns
    ///
    /// The last inserted position is returned
    fn insert_ring(&mut self, buffer_size: u64, value: ValkeyString) -> u64 {
        // Bring the cursor into the expected range
        self.set_insert_cursor(self.get_insert_cursor() % buffer_size);

        // Insert the value
        let position = self.insert(value);

        // Bring the cursor back into the expected range
        self.set_insert_cursor(self.get_insert_cursor() % buffer_size);

        position
    }

    /// Sets the value at a given position
    ///
    /// # Returns
    ///
    /// If the slot was previously unused, the function returns `1`. Otherwise `0`.
    fn set(&mut self, position: u64, value: ValkeyString) -> usize;

    /// Sets the position to insert the next item
    ///
    /// # Returns
    ///
    /// 1, if setting the position worked. 0 otherwise.
    fn set_insert_cursor(&mut self, position: u64) -> u64;

    /// Collects info about the array
    ///
    /// # Returns
    ///
    /// A [`HashMap`] with the following key/values:
    /// * `count` - number of set elements
    /// * `len` - maximum used position + 1 (0 if the array is empty)
    /// * `insert-cursor` - position of the insert cursor
    fn info(&self) -> HashMap<&'static str, String> {
        HashMap::from([
            ("count", self.count().to_string()),
            ("len", self.next_highest_position().to_string()),
            ("insert-cursor", self.get_insert_cursor().to_string()),
        ])
    }

    /// Iterates over all positions along with their values
    fn iter(&self) -> impl Iterator<Item = (&u64, &ValkeyString)>;

    /// Iterates over all positions in an array
    fn range_iter(&self, (start, end): Range) -> GenericArrayRangeIter<'_, Self> {
        GenericArrayRangeIter::new(start, end, self)
    }
}
