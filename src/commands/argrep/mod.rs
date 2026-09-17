//! Implementation of the `ARGREP` command

pub mod matchers;

use crate::Array;
use crate::array::Range;
use crate::commands::argrep::matchers::{
    ContainsMatcher, ExactMatcher, GlobMatcher, Matcher, MatchingFn, RegexMatcher,
};
use crate::commands::utils::{
    NextArgExtras, err_if_further_arguments, read_write_action, to_arg_iter,
};
use crate::registration::VKARRAY;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

/// Specification for a single match
pub enum SubstituteMatcher {
    /// Matches iff the found item matches the given one exactly
    Exact(ValkeyString),

    /// Matches iff the found item contains the given one (aka substring search)
    Contains(ValkeyString),

    /// Matches iff the found item is matched by the given glob
    ///
    /// The glob is anchored at the beginning, but not at the end.
    ///
    /// To match Valkey's glob matching, the glob gets a `*` get appended if it's not there yet.
    ///
    /// For the same reason, case-insensitive matching only converts the found items' ASCII upper
    /// case characters to lower case and matches the glob as given.
    Glob(ValkeyString),

    /// Matches iff the found item is matched by the given regular expression
    ///
    /// The regular expressions are not anchored (neither at the start nor the end).
    ///
    /// To match Valkey's glob matching, the character classes are _not_ Unicode aware.
    Regex(String),
}

impl SubstituteMatcher {
    fn get_matcher(self, case_sensitive: bool) -> ValkeyResult<MatchingFn> {
        match self {
            SubstituteMatcher::Exact(expr) => {
                ExactMatcher::new(expr).get_matcher_func(case_sensitive)
            }
            SubstituteMatcher::Contains(expr) => {
                ContainsMatcher::new(expr).get_matcher_func(case_sensitive)
            }
            SubstituteMatcher::Glob(expr) => {
                GlobMatcher::new(expr).get_matcher_func(case_sensitive)
            }
            SubstituteMatcher::Regex(expr) => {
                RegexMatcher::new(expr).get_matcher_func(case_sensitive)
            }
        }
    }
}

/// Executes the command on each position in the range (inclusive)
fn act_on_range(
    array: &mut Array,
    range: Range,
    matchers: Vec<SubstituteMatcher>,
    opt_limit: Option<u64>,
    case_sensitive: bool,
    with_values: bool,
    conjunctive: bool,
) -> ValkeyResult<Vec<ValkeyValue>> {
    let (limited, limit) = match opt_limit {
        Some(limit) => (true, limit as usize),
        None => (false, 0),
    };

    let matcher_fns = matchers
        .into_iter()
        .map(|substitute| substitute.get_matcher(case_sensitive))
        .collect::<ValkeyResult<Vec<MatchingFn>>>()?;

    let mut items = vec![];
    for (position, maybe_value) in array.range_iter(range) {
        if let Some(value) = maybe_value
            && ((conjunctive && matcher_fns.iter().all(|matcher_fn| matcher_fn(value)))
                || (!conjunctive && matcher_fns.iter().any(|matcher_fn| matcher_fn(value))))
        {
            if with_values {
                let vkpos = ValkeyValue::from(position as i64);
                let vkvalue = ValkeyValue::from(value);
                items.push(ValkeyValue::from(vec![vkpos, vkvalue]));
            } else {
                items.push((position as i64).into());
            }
        }

        // Checking an eventual limit
        if limited && items.len() == limit {
            break;
        }
    }
    Ok(items)
}

/// Implements the `ARGREP` command
pub fn argrep(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut arg_iter = to_arg_iter!(args);
    let key_name = &arg_iter.next_arg()?;
    let range = arg_iter.next_start_end()?;
    let mut opt_limit = None;
    let mut case_sensitive = true;
    let mut with_values = false;
    let mut matchers = vec![];
    let mut conjunctive = false;

    // Parse options
    while let Some(arg) = arg_iter.next() {
        match arg.to_string().to_ascii_uppercase().as_str() {
            // -- Operations ----------------
            "EXACT" => matchers.push(SubstituteMatcher::Exact(arg_iter.next_arg()?)),
            "GLOB" => matchers.push(SubstituteMatcher::Glob(arg_iter.next_arg()?)),
            "MATCH" => matchers.push(SubstituteMatcher::Contains(arg_iter.next_arg()?)),
            "RE" => {
                let raw_arg = arg_iter.next_arg()?;
                let Ok(re) = String::from_utf8(raw_arg.to_vec()) else {
                    return Err(ValkeyError::Str(
                        "ERR Regex for ARGREP is not a valid UTF-8 string",
                    ));
                };
                matchers.push(SubstituteMatcher::Regex(re));
            }

            // -- Options -------------------
            "AND" => conjunctive = true,
            "LIMIT" => opt_limit = Some(arg_iter.next_u64()?),
            "NOCASE" => case_sensitive = false,
            "OR" => conjunctive = false,
            "WITHVALUES" => with_values = true,
            _ => return Err(ValkeyError::WrongArity),
        }
    }

    err_if_further_arguments(arg_iter)?;

    if matchers.is_empty() {
        return Err(ValkeyError::WrongArity);
    }

    let items = read_write_action!(
        ctx,
        key_name,
        ValkeyValue::Array(Vec::new()),
        act_on_range,
        range,
        matchers,
        opt_limit,
        case_sensitive,
        with_values,
        conjunctive,
    )?;

    Ok(ValkeyValue::Array(items))
}

#[cfg(test)]
mod tests {

    use crate::commands::argrep;

    use crate::Array;
    use crate::commands::argrep::{SubstituteMatcher, act_on_range};
    use crate::test_utils::{u32s_to_vec_value, vkstr};
    use assertables::{assert_contains, assert_matches};
    use rstest::rstest;
    use valkey_module::test_shims::create_test_args;
    use valkey_module::{Context, ValkeyError, ValkeyValue};

    #[test]
    fn arity_too_low() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "42"]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "42", "EXACT", "bar", "baz"]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_limit() {
        let ctx = Context::test();
        let args = create_test_args(&[
            "ARGREP", "foo", "23", "42", "EXACT", "bar", "LIMIT", "4711", "baz",
        ]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_no_case() {
        let ctx = Context::test();
        let args =
            create_test_args(&["ARGREP", "foo", "23", "42", "EXACT", "bar", "NOCASE", "baz"]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_values() {
        let ctx = Context::test();
        let args = create_test_args(&[
            "ARGREP",
            "foo",
            "23",
            "42",
            "EXACT",
            "bar",
            "WITHVALUES",
            "baz",
        ]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn arity_too_high_with_all_options() {
        let ctx = Context::test();
        let args = create_test_args(&[
            "ARGREP",
            "foo",
            "23",
            "42",
            "EXACT",
            "bar",
            "LIMIT",
            "4711",
            "NOCASE",
            "WITHVALUES",
            "baz",
        ]);

        let result = argrep(&ctx, args);

        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[test]
    fn wrong_argument_type_start() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "bar", "42"]);

        let result = argrep(&ctx, args);
        let err = result.expect_err("ARGREP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_type_end() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "bar"]);

        let result = argrep(&ctx, args);
        let err = result.expect_err("ARGREP should fail");

        assert_contains!(err.to_string(), "integer");
    }

    #[test]
    fn wrong_argument_unsupported_operation() {
        let ctx = Context::test();
        let args = create_test_args(&["ARGREP", "foo", "23", "42", "bar"]);

        let result = argrep(&ctx, args);
        assert_matches!(result.unwrap_err(), ValkeyError::WrongArity);
    }

    #[rstest]
    #[case::exact_sensitive("Exact", "fOo", "sensitive", &[2])]
    #[case::exact_insensitive("Exact", "fOo", "insensitive", &[1, 2, 3, 4])]
    #[case::contains_sensitive("Contains", "Oo", "sensitive", &[2, 4])]
    #[case::contains_insensitive("Contains", "Oo", "insensitive", &[1, 2, 3, 4])]
    #[case::glob_sensitive("Glob", "f[O]*", "sensitive", &[2])]
    #[case::glob_insensitive("Glob", "f[O]*", "insensitive", &[])]
    #[case::glob_sensitive_lc("Glob", "f[o]*", "sensitive", &[1])]
    #[case::glob_insensitive_lc("Glob", "f[o]*", "insensitive", &[1, 2, 3, 4])]
    #[case::regex_sensitive("Regex", "fO+", "sensitive", &[2])]
    #[case::regex_insensitive("Regex", "fO+", "insensitive", &[1, 2, 3, 4])]
    #[case::regex_sensitive_lc("Regex", "fo+", "sensitive", &[1])]
    #[case::regex_insensitive_lc("Regex", "fo+", "insensitive", &[1, 2, 3, 4])]
    fn act_on_range_matching(
        #[case] matcher: &str,
        #[case] expr: &str,
        #[case] case_sensitivity: &str,
        #[case] expected_idxs_from_1_to_4: &[i32],
        #[values(true, false)] limited: bool,
        #[values(true, false)] with_values: bool,
    ) {
        let values = [
            (0, "foo"), // ignored (not in range)
            // ----
            (1, "foo"), // All case combinations for first two letters
            (2, "fOo"),
            (3, "Foo"),
            (4, "FOo"),
            (5, "foo"), // All case combinations again, so we can test limits
            (6, "fOo"),
            (7, "Foo"),
            (8, "FOo"),
            (9, "bar"),  // different value to detect if we match too much
            (10, "foo"), // ignored (not in range)
        ];

        // Building the array to match against
        let mut array = Array::new();
        for (pos, value) in values {
            array.set(&(pos as u64), &vkstr(value));
        }

        // Building the match parameters
        let matcher_param = vec![match matcher {
            "Contains" => SubstituteMatcher::Contains(vkstr(expr)),
            "Exact" => SubstituteMatcher::Exact(vkstr(expr)),
            "Glob" => SubstituteMatcher::Glob(vkstr(expr)),
            "Regex" => SubstituteMatcher::Regex(expr.to_string()),
            _ => panic!("Unknown matcher: {matcher}"),
        }];
        let case_sensitivity_param = !case_sensitivity.starts_with("in");
        let limited_param = if limited { Some(1) } else { None };

        // Performing the match
        let result = act_on_range(
            &mut array,
            (1, 9),
            matcher_param,
            limited_param,
            case_sensitivity_param,
            with_values,
            false,
        )
        .unwrap();

        // Building the expected value
        let idx_iter = expected_idxs_from_1_to_4
            .iter()
            .copied()
            .chain(expected_idxs_from_1_to_4.iter().map(|x| (*x) + 4));
        let mut expected: Vec<ValkeyValue> = if with_values {
            idx_iter
                .map(|x| {
                    let pos = ValkeyValue::from(x as i64);
                    let value = vkstr(values[x as usize].1).into();
                    vec![pos, value].into()
                })
                .collect()
        } else {
            idx_iter.map(|x| (x as i64).into()).collect()
        };
        if limited {
            expected.truncate(1);
        }

        // And finally, the check
        assert_eq!(result, expected);
    }

    #[test]
    fn act_on_range_multiple_matchers_disjunctive() {
        // Building the array to match against
        let mut array = Array::new();
        array.set(&0, &vkstr("foobar"));
        array.set(&1, &vkstr("foo"));
        array.set(&2, &vkstr("bar"));
        array.set(&3, &vkstr("baz"));
        array.set(&4, &vkstr("quux"));
        let matcher = vec![
            SubstituteMatcher::Contains(vkstr("foo")),
            SubstituteMatcher::Exact(vkstr("bar")),
            SubstituteMatcher::Exact(vkstr("quux")),
        ];
        let result = act_on_range(&mut array, (0, 4), matcher, None, true, false, false).unwrap();

        assert_eq!(result, u32s_to_vec_value(&[0, 1, 2, 4]))
    }

    #[test]
    fn act_on_range_multiple_matchers_conjunctive() {
        // Building the array to match against
        let mut array = Array::new();
        array.set(&0, &vkstr("foobar"));
        array.set(&1, &vkstr("barfoo"));
        array.set(&2, &vkstr("foobaz"));
        array.set(&3, &vkstr("bazquuxfoo"));
        array.set(&4, &vkstr("bafooz"));
        array.set(&5, &vkstr("foo"));
        array.set(&6, &vkstr("bar"));
        array.set(&7, &vkstr("baz"));
        array.set(&8, &vkstr("quux"));
        let matcher = vec![
            SubstituteMatcher::Contains(vkstr("foo")),
            SubstituteMatcher::Contains(vkstr("ba")),
            SubstituteMatcher::Contains(vkstr("z")),
        ];
        let result = act_on_range(&mut array, (0, 8), matcher, None, true, false, true).unwrap();

        assert_eq!(result, u32s_to_vec_value(&[2, 3, 4]))
    }
}
