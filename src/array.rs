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

    /// Gets the position where the next item will get inserted
    pub fn get_insert_cursor(&self) -> u64 {
        self.insert_cursor
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

    /// Inserts an element in ring-buffer fashion
    ///
    /// # Returns
    ///
    /// The last inserted position is returned
    pub fn insert_ring(&mut self, buffer_size: u64, value: &ValkeyString) -> u64 {
        // Bring the cursor into the expected range
        self.insert_cursor %= buffer_size;

        // Insert the value
        let position = self.insert(value);

        // Bring the cursor back into the expected range
        self.insert_cursor %= buffer_size;

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

    /// Sets the position to insert the next item
    ///
    /// # Returns
    ///
    /// 1, if setting the position worked. 0 otherwise.
    pub fn set_insert_cursor(&mut self, position: u64) -> u64 {
        self.insert_cursor = position;
        1
    }

    /// Collects info about the array
    ///
    /// # Returns
    ///
    /// A [`HashMap`] with the following key/values:
    /// * `count` - number of set elements
    /// * `len` - maximum used position + 1 (0 if the array is empty)
    /// * `insert-cursor` - position of the insert cursor
    pub fn info(&self) -> HashMap<&'static str, String> {
        HashMap::from([
            ("count", self.count().to_string()),
            ("len", self.next_highest_position.to_string()),
            ("insert-cursor", self.insert_cursor.to_string()),
        ])
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
        use crate::test_utils::vkstr;

        pub fn assert_array_entry<S: Into<String>>(array: &Array, position: u64, expected: S) {
            let value = array
                .get(&position)
                .unwrap_or_else(|| panic!("array should have a value at {position}"));

            let expected_str = vkstr(expected);
            assert_eq!(*value, *expected_str, "\"{value}\" == \"{expected_str}\"");
        }

        pub fn assert_array_no_entry(array: &Array, position: u64) {
            if let Some(entry) = array.get(&position) {
                panic!("array should be empty but is \"{entry}\" at {position}");
            }
        }
    }

    use crate::Array;
    use crate::array::tests::util::{assert_array_entry, assert_array_no_entry};
    use crate::test_utils::vkstr;
    use std::collections::HashMap;

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

    #[test]
    fn array_insert_ring() {
        let mut array = Array::new();

        // We test with a ring buffer of size three. So the fourth element should overwrite the
        // first.

        // Insert first item
        let res = array.insert_ring(3, &vkstr("foo"));
        assert_eq!(res, 0);

        // Insert second item
        let res = array.insert_ring(3, &vkstr("bar"));
        assert_eq!(res, 1);

        // Insert third item
        let res = array.insert_ring(3, &vkstr("baz"));
        assert_eq!(res, 2);

        // Insert fourth item. This should overwrite the first
        let res = array.insert_ring(3, &vkstr("quux"));
        assert_eq!(res, 0);

        assert_array_entry(&array, 0, "quux");
        assert_array_entry(&array, 1, "bar");
        assert_array_entry(&array, 2, "baz");

        // Check that the insert cursor got updated accordingly
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);
    }

    #[test]
    fn array_insert_ring_initial_clamp() {
        let mut array = Array::new();

        // We set the insert cursor to 42
        array.set_insert_cursor(42);

        // Then treat it as ring buffer of size 23 and insert an element, which should land at
        // position 19 (= 42 % 23)
        array.insert_ring(23, &vkstr("foo"));

        assert_array_entry(&array, 19, "foo");
        assert_array_no_entry(&array, 42);

        // Check that the insert cursor got updated accordingly
        let res = array.get_insert_cursor();
        assert_eq!(res, 20);
    }

    #[test]
    fn array_get_insert_cursor() {
        let mut array = Array::new();

        // Initial cursor on fresh Array
        let res = array.get_insert_cursor();
        assert_eq!(res, 0);

        // First insert
        array.insert(&vkstr("foo"));
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);

        // Deleting it again does not change the cursor
        array.del(&0);
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);

        // Set a value at a higher position to check that it does not influence the cursor
        array.set(&42, &vkstr("bar"));
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);

        // Final insertion to check that we're not stuck at 1
        array.insert(&vkstr("foo"));
        let res = array.get_insert_cursor();
        assert_eq!(res, 2);
    }

    #[test]
    fn array_set_insert_cursor() {
        let mut array = Array::new();

        // Setting the inser cursor on fresh Array
        let res = array.set_insert_cursor(42);
        assert_eq!(res, 1);
        let res = array.get_insert_cursor();
        assert_eq!(res, 42);
    }

    #[test]
    fn array_info() {
        let mut array = Array::new();

        // Checking on an empty Array
        let info = array.info();
        let expected = HashMap::from([
            ("count", "0".to_string()),
            ("len", "0".to_string()),
            ("insert-cursor", "0".to_string()),
        ]);
        assert_eq!(info, expected);

        // Setting some data in the array
        array.set(&42, &vkstr("bar"));
        array.set(&23, &vkstr("baz"));
        array.set_insert_cursor(4711);

        // Checking info again
        let info = array.info();
        let expected = HashMap::from([
            ("count", "2".to_string()),
            ("len", "43".to_string()),
            ("insert-cursor", "4711".to_string()),
        ]);
        assert_eq!(info, expected);
    }
}
