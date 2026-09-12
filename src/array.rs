//! The Rust implementation of Valkey arrays
//!
//! This gets registered in Valkey via [`crate::registration::VKARRAY`].

use std::collections::HashMap;
use valkey_module::ValkeyString;

/// The struct that models the data in Rust
#[derive(Default, Debug)]
pub struct Array {
    /// The array's value as map
    values: HashMap<u64, ValkeyString>,

    /// The position where the next inserted element will land
    insert_cursor: u64,

    /// The maximum used position + 1 (0 if the array is empty)
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
    pub fn del(&mut self, position: &u64) -> u64 {
        let count = match self.values.remove(position) {
            Some(_) => 1,
            None => 0,
        };

        // If we deleted `next_highest_position`, recompute it
        if position + 1 == self.next_highest_position {
            self.next_highest_position = match self.values.keys().max() {
                Some(max) => max + 1,
                None => 0,
            }
        }

        count
    }

    /// Gets the value at a given position
    // Returns an owned value instead of a reference as `ARGET` needs an owned value anyways.
    pub fn get(&self, position: &u64) -> Option<ValkeyString> {
        self.values.get(position).cloned()
    }

    /// Gets the value at a given position
    // Returns an owned value instead of a reference as `ARGET` needs an owned value anyways.
    pub fn insert(&mut self, value: &ValkeyString) -> u64 {
        // Inserting the value
        let position = self.insert_cursor;
        self.set(&position, value);

        // Bumping the insert cursor
        self.insert_cursor += 1;

        // Return the inserted position
        position
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

    /// Iterates over all positions along with their values
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &ValkeyString)> {
        self.values.iter()
    }
}

#[cfg(test)]
mod tests {
    mod util {
        use crate::Array;
        use valkey_module::ValkeyString;

        pub fn vkstr<S: Into<String>>(input: S) -> ValkeyString {
            ValkeyString::test(input.into())
        }

        #[allow(clippy::panic, reason = "assertions are allowed to crash out")]
        pub fn assert_array_entry<S: Into<String>>(array: &Array, position: u64, expected: S) {
            let value = array
                .get(&position)
                .unwrap_or_else(|| panic!("array should have a value at {position}"));

            let expected_str = vkstr(expected);
            assert_eq!(*value, *expected_str, "\"{value}\" == \"{expected_str}\"");
        }

        #[allow(clippy::panic, reason = "assertions are allowed to crash out")]
        pub fn assert_array_no_entry(array: &Array, position: u64) {
            if let Some(entry) = array.get(&position) {
                panic!("array should be empty but is \"{entry}\" at {position}");
            }
        }
    }

    use crate::Array;
    use crate::array::tests::util::{assert_array_entry, assert_array_no_entry, vkstr};

    #[test]
    fn array_basic_get_set() {
        let mut array = Array::new();

        // No entries in the empty array
        assert_array_no_entry(&array, 23);
        assert_array_no_entry(&array, 42);
        assert_array_no_entry(&array, 4711);

        // Adding a single slot
        let added_slots = array.set(&23, &vkstr("foo"));
        assert_eq!(added_slots, 1); // New slot, as it was empty before

        assert_array_entry(&array, 23, "foo");
        assert_array_no_entry(&array, 42);
        assert_array_no_entry(&array, 4711);

        // Adding a different slot
        let added_slots = array.set(&42, &vkstr("bar"));
        assert_eq!(added_slots, 1);

        assert_array_entry(&array, 23, "foo");
        assert_array_entry(&array, 42, "bar");
        assert_array_no_entry(&array, 4711);

        // Adding to first slot again
        let added_slots = array.set(&23, &vkstr("baz"));
        assert_eq!(added_slots, 0); // No new slot, as it was occupied before

        assert_array_entry(&array, 23, "baz");
        assert_array_entry(&array, 42, "bar");
        assert_array_no_entry(&array, 4711);
    }

    #[test]
    fn array_del() {
        let mut array = Array::new();

        // Deleting an unused slot
        let deleted = array.del(&42);
        assert_eq!(deleted, 0);

        // Setting a slot and deleting it again
        array.set(&42, &vkstr("bar"));
        let deleted = array.del(&42);
        assert_eq!(deleted, 1);
    }

    #[test]
    fn array_count() {
        let mut array = Array::new();

        assert_eq!(array.count(), 0);

        array.set(&42, &vkstr("bar"));
        assert_eq!(array.count(), 1);

        array.set(&23, &vkstr("baz"));
        assert_eq!(array.count(), 2);
    }

    #[test]
    fn array_next_highest_position() {
        let mut array = Array::new();

        assert_eq!(array.next_highest_position(), 0);

        array.set(&42, &vkstr("bar"));
        assert_eq!(array.next_highest_position(), 43);

        array.set(&23, &vkstr("baz"));
        assert_eq!(array.next_highest_position(), 43);
    }

    #[test]
    fn array_next_highest_position_after_deletion() {
        let mut array = Array::new();

        array.set(&23, &vkstr("baz"));
        array.set(&42, &vkstr("bar"));

        array.del(&42);
        assert_eq!(array.next_highest_position(), 24);

        array.del(&23);
        assert_eq!(array.next_highest_position(), 0);
    }

    #[test]
    fn array_insert() {
        let mut array = Array::new();

        // First insert
        let res = array.insert(&vkstr("foo"));
        assert_eq!(res, 0);

        assert_array_entry(&array, 0, "foo");
        assert_array_no_entry(&array, 1);
        assert_array_no_entry(&array, 2);

        // Second insert
        let res = array.insert(&vkstr("bar"));
        assert_eq!(res, 1);

        assert_array_entry(&array, 0, "foo");
        assert_array_entry(&array, 1, "bar");
        assert_array_no_entry(&array, 2);

        // Insert after higher set
        array.set(&42, &vkstr("baz"));

        let res = array.insert(&vkstr("quux"));
        assert_eq!(res, 2);

        assert_array_entry(&array, 0, "foo");
        assert_array_entry(&array, 1, "bar");
        assert_array_entry(&array, 2, "quux");
    }
}
