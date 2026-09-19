//! Array implemented by a single [`HashMap`]
//!
use crate::array::{ArrayType, collect_generic_info};
use std::collections::HashMap;
use valkey_module::ValkeyString;

/// Array implemented by a single [`HashMap`]
#[derive(Default, Debug)]
pub struct SingleHashMapArray {
    /// The array's value as map
    values: HashMap<u64, ValkeyString>,

    /// The position where the next inserted element will land
    insert_cursor: u64,

    /// The maximum used position + 1 (0 if the array is empty)
    next_highest_position: u64,
}

impl SingleHashMapArray {
    /// Checks if the array is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
impl ArrayType for SingleHashMapArray {
    fn type_name() -> &'static str {
        "SingleHashMap"
    }

    fn new() -> Self {
        Self::default()
    }

    /// Gets the number of entries
    fn count(&self) -> usize {
        self.values.len()
    }

    fn next_highest_position(&self) -> u64 {
        self.next_highest_position
    }

    fn del(&mut self, position: u64) -> u64 {
        let count = match self.values.remove(&position) {
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

    fn get(&self, position: u64) -> Option<&ValkeyString> {
        self.values.get(&position)
    }

    fn get_insert_cursor(&self) -> u64 {
        self.insert_cursor
    }

    fn insert(&mut self, value: ValkeyString) -> u64 {
        // Inserting the value
        let position = self.insert_cursor;
        self.set(position, value);

        // Bumping the insert cursor
        self.insert_cursor += 1;

        // Return the inserted position
        position
    }

    fn insert_ring(&mut self, buffer_size: u64, value: ValkeyString) -> u64 {
        // Bring the cursor into the expected range
        self.insert_cursor %= buffer_size;

        // Insert the value
        let position = self.insert(value);

        // Bring the cursor back into the expected range
        self.insert_cursor %= buffer_size;

        position
    }

    fn set(&mut self, position: u64, value: ValkeyString) -> u64 {
        self.next_highest_position = self.next_highest_position.max(position + 1);
        if self.values.insert(position, value).is_some() {
            // The position already had a value, so it's not a new slot
            0
        } else {
            // The position previously did not have a value, so it's a new slot
            1
        }
    }

    fn set_insert_cursor(&mut self, position: u64) -> u64 {
        self.insert_cursor = position;
        1
    }

    fn info(&self, full: bool) -> HashMap<&'static str, String> {
        let mut ret = collect_generic_info(self);

        if full {
            ret.insert("capacity", self.values.capacity().to_string());
        }

        ret
    }

    fn iter(&self) -> impl Iterator<Item = (&u64, &ValkeyString)> {
        self.values.iter()
    }
}

#[cfg(test)]
mod tests {
    mod util {
        use super::SingleHashMapArray;
        use crate::array::ArrayType;
        use crate::test_utils::vkstr;

        pub fn assert_array_entry<S: Into<String>>(
            array: &SingleHashMapArray,
            position: u64,
            expected: S,
        ) {
            let value = array
                .get(position)
                .unwrap_or_else(|| panic!("array should have a value at {position}"));

            let expected_str = vkstr(expected);
            assert_eq!(*value, expected_str, "\"{value}\" == \"{expected_str}\"");
        }

        pub fn assert_array_no_entry(array: &SingleHashMapArray, position: u64) {
            if let Some(entry) = array.get(position) {
                panic!("array should be empty but is \"{entry}\" at {position}");
            }
        }
    }

    use super::SingleHashMapArray;
    use crate::array::ArrayType;
    use crate::test_utils::vkstr;
    use std::collections::HashMap;
    use util::{assert_array_entry, assert_array_no_entry};

    #[test]
    fn array_basic_get_set() {
        let mut array = SingleHashMapArray::new();

        // No entries in the empty array
        assert_array_no_entry(&array, 23);
        assert_array_no_entry(&array, 42);
        assert_array_no_entry(&array, 4711);

        // Adding a single slot
        let added_slots = array.set(23, vkstr("foo"));
        assert_eq!(added_slots, 1); // New slot, as it was empty before

        assert_array_entry(&array, 23, "foo");
        assert_array_no_entry(&array, 42);
        assert_array_no_entry(&array, 4711);

        // Adding a different slot
        let added_slots = array.set(42, vkstr("bar"));
        assert_eq!(added_slots, 1);

        assert_array_entry(&array, 23, "foo");
        assert_array_entry(&array, 42, "bar");
        assert_array_no_entry(&array, 4711);

        // Adding to first slot again
        let added_slots = array.set(23, vkstr("baz"));
        assert_eq!(added_slots, 0); // No new slot, as it was occupied before

        assert_array_entry(&array, 23, "baz");
        assert_array_entry(&array, 42, "bar");
        assert_array_no_entry(&array, 4711);
    }

    #[test]
    fn array_del() {
        let mut array = SingleHashMapArray::new();

        // Deleting an unused slot
        let deleted = array.del(42);
        assert_eq!(deleted, 0);

        // Setting a slot and deleting it again
        array.set(42, vkstr("bar"));
        let deleted = array.del(42);
        assert_eq!(deleted, 1);
    }

    #[test]
    fn array_count() {
        let mut array = SingleHashMapArray::new();

        assert_eq!(array.count(), 0);

        array.set(42, vkstr("bar"));
        assert_eq!(array.count(), 1);

        array.set(23, vkstr("baz"));
        assert_eq!(array.count(), 2);
    }

    #[test]
    fn array_next_highest_position() {
        let mut array = SingleHashMapArray::new();

        assert_eq!(array.next_highest_position(), 0);

        array.set(42, vkstr("bar"));
        assert_eq!(array.next_highest_position(), 43);

        array.set(23, vkstr("baz"));
        assert_eq!(array.next_highest_position(), 43);
    }

    #[test]
    fn array_next_highest_position_after_deletion() {
        let mut array = SingleHashMapArray::new();

        array.set(23, vkstr("baz"));
        array.set(42, vkstr("bar"));

        array.del(42);
        assert_eq!(array.next_highest_position(), 24);

        array.del(23);
        assert_eq!(array.next_highest_position(), 0);
    }

    #[test]
    fn array_insert() {
        let mut array = SingleHashMapArray::new();

        // First insert
        let res = array.insert(vkstr("foo"));
        assert_eq!(res, 0);

        assert_array_entry(&array, 0, "foo");
        assert_array_no_entry(&array, 1);
        assert_array_no_entry(&array, 2);

        // Second insert
        let res = array.insert(vkstr("bar"));
        assert_eq!(res, 1);

        assert_array_entry(&array, 0, "foo");
        assert_array_entry(&array, 1, "bar");
        assert_array_no_entry(&array, 2);

        // Insert after higher set
        array.set(42, vkstr("baz"));

        let res = array.insert(vkstr("quux"));
        assert_eq!(res, 2);

        assert_array_entry(&array, 0, "foo");
        assert_array_entry(&array, 1, "bar");
        assert_array_entry(&array, 2, "quux");
    }

    #[test]
    fn array_insert_ring() {
        let mut array = SingleHashMapArray::new();

        // We test with a ring buffer of size three. So the fourth element should overwrite the
        // first.

        // Insert first item
        let res = array.insert_ring(3, vkstr("foo"));
        assert_eq!(res, 0);

        // Insert second item
        let res = array.insert_ring(3, vkstr("bar"));
        assert_eq!(res, 1);

        // Insert third item
        let res = array.insert_ring(3, vkstr("baz"));
        assert_eq!(res, 2);

        // Insert fourth item. This should overwrite the first
        let res = array.insert_ring(3, vkstr("quux"));
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
        let mut array = SingleHashMapArray::new();

        // We set the insert cursor to 42
        array.set_insert_cursor(42);

        // Then treat it as ring buffer of size 23 and insert an element, which should land at
        // position 19 (= 42 % 23)
        array.insert_ring(23, vkstr("foo"));

        assert_array_entry(&array, 19, "foo");
        assert_array_no_entry(&array, 42);

        // Check that the insert cursor got updated accordingly
        let res = array.get_insert_cursor();
        assert_eq!(res, 20);
    }

    #[test]
    fn array_get_insert_cursor() {
        let mut array = SingleHashMapArray::new();

        // Initial cursor on fresh Array
        let res = array.get_insert_cursor();
        assert_eq!(res, 0);

        // First insert
        array.insert(vkstr("foo"));
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);

        // Deleting it again does not change the cursor
        array.del(0);
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);

        // Set a value at a higher position to check that it does not influence the cursor
        array.set(42, vkstr("bar"));
        let res = array.get_insert_cursor();
        assert_eq!(res, 1);

        // Final insertion to check that we're not stuck at 1
        array.insert(vkstr("foo"));
        let res = array.get_insert_cursor();
        assert_eq!(res, 2);
    }

    #[test]
    fn array_set_insert_cursor() {
        let mut array = SingleHashMapArray::new();

        // Setting the inser cursor on fresh Array
        let res = array.set_insert_cursor(42);
        assert_eq!(res, 1);
        let res = array.get_insert_cursor();
        assert_eq!(res, 42);
    }

    #[test]
    fn array_info() {
        let mut array = SingleHashMapArray::new();

        // Checking on an empty Array
        let info = array.info(false);
        let expected = HashMap::from([
            ("type", "SingleHashMap".to_string()),
            ("count", "0".to_string()),
            ("len", "0".to_string()),
            ("insert-cursor", "0".to_string()),
        ]);
        assert_eq!(info, expected);

        // Setting some data in the array
        array.set(42, vkstr("bar"));
        array.set(23, vkstr("baz"));
        array.set_insert_cursor(4711);

        // Checking info again
        let info = array.info(false);
        let expected = HashMap::from([
            ("type", "SingleHashMap".to_string()),
            ("count", "2".to_string()),
            ("len", "43".to_string()),
            ("insert-cursor", "4711".to_string()),
        ]);
        assert_eq!(info, expected);
    }

    #[test]
    fn array_info_full() {
        let array = SingleHashMapArray::new();

        // Checking on an empty Array
        let info = array.info(true);
        let expected = HashMap::from([
            ("type", "SingleHashMap".to_string()),
            ("count", "0".to_string()),
            ("len", "0".to_string()),
            ("insert-cursor", "0".to_string()),
            ("capacity", "0".to_string()),
        ]);
        assert_eq!(info, expected);
    }

    #[test]
    fn iterator_empty_array() {
        let array = SingleHashMapArray::new();

        // normal range
        let items = array.range_iter((40, 42)).collect::<Vec<_>>();
        assert_eq!(items, &[(40, None), (41, None), (42, None)]);
    }

    #[test]
    fn iterator_single_element_array() {
        let mut array = SingleHashMapArray::new();
        array.set(42, vkstr("foo"));

        // covering range, item in the middle
        let items = array.range_iter((41, 43)).collect::<Vec<_>>();
        assert_eq!(items, &[(41, None), (42, Some(&vkstr("foo"))), (43, None)]);

        // covering range, item at start
        let items = array.range_iter((42, 44)).collect::<Vec<_>>();
        assert_eq!(items, &[(42, Some(&vkstr("foo"))), (43, None), (44, None)]);

        // covering range, item at end
        let items = array.range_iter((40, 42)).collect::<Vec<_>>();
        assert_eq!(items, &[(40, None), (41, None), (42, Some(&vkstr("foo")))]);

        // range before item
        let items = array.range_iter((39, 41)).collect::<Vec<_>>();
        assert_eq!(items, &[(39, None), (40, None), (41, None)]);

        // range after item
        let items = array.range_iter((43, 45)).collect::<Vec<_>>();
        assert_eq!(items, &[(43, None), (44, None), (45, None)]);

        // range is item
        let items = array.range_iter((42, 42)).collect::<Vec<_>>();
        assert_eq!(items, &[(42, Some(&vkstr("foo")))]);
    }

    #[test]
    fn iterator_multiple_elements_array() {
        let mut array = SingleHashMapArray::new();
        array.set(42, vkstr("foo"));
        // No element at 43
        array.set(44, vkstr("bar"));
        array.set(45, vkstr("baz"));

        let items = array.range_iter((41, 46)).collect::<Vec<_>>();
        assert_eq!(
            items,
            &[
                (41, None),
                (42, Some(&vkstr("foo"))),
                (43, None),
                (44, Some(&vkstr("bar"))),
                (45, Some(&vkstr("baz"))),
                (46, None)
            ]
        );
    }
    #[test]
    fn is_empty() {
        let mut array = SingleHashMapArray::new();

        // It's initially empty
        assert!(array.is_empty());

        // Adding an element makes it non-empty
        array.set(42, vkstr("foo"));
        assert!(!array.is_empty());

        // Removing the element again makes it again empty
        array.del(42);
        assert!(array.is_empty());
    }
}
