//! Implementation of the `ARCOUNT` command

use crate::Array;
use crate::array::{ArrayType, Range};
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_only_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Executes the command on the given args
fn act_on_args(array: &Array, count: u64, reverse: bool) -> Vec<ValkeyValue> {
    let cursor = array.get_insert_cursor();
    // Bail out, if there's nothing to do
    if count == 0 {
        return vec![];
    }

    // Bail out, if we cannot go back from the cursor
    if cursor == 0 {
        return vec![];
    }

    let end = cursor - 1;
    let start = end - (count - 1).min(end);
    let range = Range::new_maybe_rev(start, end, reverse);

    array
        .range_iter(range)
        .map(|(_, maybe_val)| match maybe_val {
            Some(val) => val.into(),
            None => ValkeyValue::Null,
        })
        .collect::<Vec<_>>()
}

/// Implements the `ARLASTITEMS` command
pub fn arlastitems(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    // As the count relates to the indices, we parse it as position
    let count = arg_iter.next_position()?;
    let mut reverse = false;

    // Parse optional limit
    if let Some(arg) = arg_iter.next() {
        match arg.to_string().to_ascii_uppercase().as_str() {
            "REV" => reverse = true,
            _ => return Err(ValkeyError::WrongArity),
        }
    }

    err_if_further_arguments(arg_iter)?;

    let items = read_only_action!(
        ctx,
        key_name,
        ValkeyValue::Integer(0),
        act_on_args,
        count,
        reverse
    );

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {
    mod util {
        use crate::Array;
        use crate::array::ArrayType;
        use crate::utils::test_utils::vkstr;

        /// Builds a new array with the entries `foo`, `bar`, and `baz`.
        pub fn new_foo_bar_baz_array() -> Array {
            let mut array = Array::new();

            array.insert(vkstr("foo"));
            array.insert(vkstr("bar"));
            array.insert(vkstr("baz"));

            array
        }
    }
    use crate::commands::arlastitems;
    use crate::utils::test_utils::{assert_arity_error, strs_to_vec_value};

    use crate::array::ArrayType;
    use crate::commands::arlastitems::act_on_args;
    use crate::commands::arlastitems::tests::util::new_foo_bar_baz_array;
    use valkey_module::Context;
    use valkey_module::test_shims::create_test_args;

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARLASTITEMS"]);

        let result = arlastitems(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high_without_rev() {
        let ctx = Context::test();
        let args = create_test_args(&["ARLASTITEMS", "foo", "42", "baz"]);

        let result = arlastitems(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn arity_too_high_with_rev() {
        let ctx = Context::test();
        let args = create_test_args(&["ARLASTITEMS", "foo", "42", "REV", "baz"]);

        let result = arlastitems(&ctx, args);

        assert_arity_error(result);
    }

    #[test]
    fn act_on_args_simple() {
        let array = new_foo_bar_baz_array();

        let items = act_on_args(&array, 2, false);
        assert_eq!(items, strs_to_vec_value(&["bar", "baz"]));
    }

    #[test]
    fn act_on_args_count_equals_length() {
        let array = new_foo_bar_baz_array();

        let items = act_on_args(&array, 3, false);
        assert_eq!(items, strs_to_vec_value(&["foo", "bar", "baz"]));
    }

    #[test]
    fn act_on_args_count_longer_than_length() {
        let array = new_foo_bar_baz_array();

        let items = act_on_args(&array, 5, false);
        assert_eq!(items, strs_to_vec_value(&["foo", "bar", "baz"]));
    }

    #[test]
    fn act_on_args_count_zero() {
        let array = new_foo_bar_baz_array();

        let items = act_on_args(&array, 0, false);
        assert_eq!(items, vec![]);
    }

    #[test]
    fn act_on_args_deleted_mid() {
        let mut array = new_foo_bar_baz_array();
        array.del(1);

        let items = act_on_args(&array, 3, false);
        assert_eq!(items, strs_to_vec_value(&["foo", "", "baz"]));
    }

    #[test]
    fn act_on_args_deleted_start() {
        let mut array = new_foo_bar_baz_array();
        array.del(0);

        let items = act_on_args(&array, 3, false);
        assert_eq!(items, strs_to_vec_value(&["", "bar", "baz"]));
    }

    #[test]
    fn act_on_args_deleted_end() {
        let mut array = new_foo_bar_baz_array();
        array.del(2);

        let items = act_on_args(&array, 3, false);
        assert_eq!(items, strs_to_vec_value(&["foo", "bar", ""]));
    }

    #[test]
    fn act_on_args_after_seek() {
        let mut array = new_foo_bar_baz_array();
        array.set_insert_cursor(5);

        let items = act_on_args(&array, 4, false);
        assert_eq!(items, strs_to_vec_value(&["bar", "baz", "", ""]));
    }

    #[test]
    fn act_on_args_mix() {
        let mut array = new_foo_bar_baz_array();
        array.del(1);
        array.set_insert_cursor(5);

        let items = act_on_args(&array, 4, true);
        assert_eq!(items, strs_to_vec_value(&["", "", "baz", ""]));
    }
}
