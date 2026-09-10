//! The Rust implementation of Valkey arrays
//!
//! This gets registered in Valkey via [`crate::registration::VKARRAY`].

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
        let res = self.values.remove(position);

        // If we deleted `next_highest_position`, recompute it
        if position + 1 == self.next_highest_position {
            self.next_highest_position = match self.values.keys().max() {
                Some(max) => max + 1,
                None => 0,
            }
        }
        res
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

    /// Iterates over all positions along with their values
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &ValkeyString)> {
        self.values.iter()
    }
}

#[cfg(test)]
mod tests {
    mod util {
        use valkey_module::ValkeyString;
        use crate::Array;

        pub fn vkstr<S: Into<String>>(input: S) -> ValkeyString {
            ValkeyString::test(input.into())
        }

        pub fn assert_array_entry<S: Into<String>>(array: &Array, position: u64, expected: S) {
            let value =array.get(&position).unwrap_or_else(|| panic!("array should have a value at {position}"));

            let expected_str = vkstr(expected);
            assert_eq!(*value, expected_str, "\"{}\" == \"{}\"", value.to_string(), expected_str);
        }

        pub fn assert_array_no_entry(array: &Array, position: u64) {
            if let Some(entry) = array.get(&position) {
                panic!("array should be empty but is \"{}\" at {position}", entry.to_string());
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
        assert!(array.del(&42).is_none());

        // Setting a slot and deleting it again
        array.set(&42, &vkstr("bar"));
        let deleted = array.del(&42).expect("del should yield the old value");
        assert_eq!(deleted, vkstr("bar"));
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

        array.del(&42).expect("del should yield the old value");
        assert_eq!(array.next_highest_position(), 24);

        array.del(&23).expect("del should yield the old value");
        assert_eq!(array.next_highest_position(), 0);
    }
}