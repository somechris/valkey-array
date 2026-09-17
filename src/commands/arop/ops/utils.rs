//! Utility functions for `AROP` operations

use valkey_module::ValkeyString;

pub fn vkstring_to_floored_i64(input: &ValkeyString) -> Result<i64, &'static str> {
    let Ok(input_str) = input.try_as_str() else {
        return Err("value is not UTF-8");
    };

    if let Some(integer_str) = input_str.split('.').next() {
        integer_str
            .parse::<i64>()
            .map_err(|_| "value is not an integer")
    } else {
        Err("value is not UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::vkstring_to_floored_i64;
    use crate::test_utils::vkstr;
    use assertables::assert_err;

    #[test]
    fn vkstring_to_i64_non_numbers_err() {
        assert_err!(vkstring_to_floored_i64(&vkstr("")));
        assert_err!(vkstring_to_floored_i64(&vkstr("foo")));
    }

    #[test]
    fn vkstring_to_i64_i64_works() {
        assert_eq!(
            vkstring_to_floored_i64(&vkstr(i64::MIN.to_string())).unwrap(),
            i64::MIN
        );
        assert_eq!(vkstring_to_floored_i64(&vkstr("4711")).unwrap(), 4711);
        assert_eq!(
            vkstring_to_floored_i64(&vkstr(i64::MAX.to_string())).unwrap(),
            i64::MAX
        );
    }

    #[test]
    fn vkstring_to_i64_f64_works() {
        let mut value = i64::MIN.to_string();
        value.push_str(".99");
        assert_eq!(vkstring_to_floored_i64(&vkstr(value)).unwrap(), i64::MIN);

        assert_eq!(
            vkstring_to_floored_i64(&vkstr("4711.0000000000000000001")).unwrap(),
            4711
        );
        assert_eq!(
            vkstring_to_floored_i64(&vkstr("4711.9999999999999999999")).unwrap(),
            4711
        );
        assert_eq!(
            vkstring_to_floored_i64(&vkstr("-4711.0000000000000000001")).unwrap(),
            -4711
        );
        assert_eq!(
            vkstring_to_floored_i64(&vkstr("-4711.9999999999999999999")).unwrap(),
            -4711
        );

        let mut value = i64::MAX.to_string();
        value.push_str(".99");
        assert_eq!(vkstring_to_floored_i64(&vkstr(value)).unwrap(), i64::MAX);
    }
}
