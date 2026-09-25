use crate::commands::argrep::matchers::{Matcher, MatchingFn};
use regex::bytes::RegexBuilder;
use valkey_module::{ValkeyError, ValkeyResult};

/// [`Matcher`] that matches only iff the candidate and the search expression are the same
pub struct RegexMatcher {
    search_expr: String,
}

impl RegexMatcher {
    /// Builds a new matcher for the given search expression
    pub fn new(search_expr: String) -> Self {
        Self { search_expr }
    }
}

impl Matcher for RegexMatcher {
    fn get_matcher_func_case_sensitive(self) -> ValkeyResult<MatchingFn> {
        let Ok(regex) = RegexBuilder::new(&self.search_expr).build() else {
            return Err(ValkeyError::Str("not a valid regular expression"));
        };
        Ok(Box::new(move |candidate| regex.is_match(candidate)))
    }

    fn get_matcher_func_case_insensitive(self) -> ValkeyResult<MatchingFn> {
        let Ok(regex) = RegexBuilder::new(&self.search_expr)
            .case_insensitive(true)
            .build()
        else {
            return Err(ValkeyError::Str("not a valid regular expression"));
        };
        Ok(Box::new(move |candidate| regex.is_match(candidate)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Matcher, RegexMatcher};

    use crate::utils::test_utils::vkstr;

    #[test]
    fn sensitive() {
        fn check(text: &str, re: &str) -> bool {
            RegexMatcher::new(re.to_string())
                .get_matcher_func_case_sensitive()
                .unwrap()(&vkstr(text))
        }

        assert!(check("foo", "foo")); // full match
        assert!(check("foo", "oo")); // unanchored start
        assert!(check("foo", "fo")); // unanchored end
        assert!(check("foo", "fo*x*o")); // regex with arbirary repetitions
        assert!(check("foo", "fo+o")); // regex with at least one
        assert!(check("foo", "fo?oo")); // regex with optional
        assert!(check("foo", "^foo")); // regex with start matching
        assert!(check("foo", "foo$")); // regex with end matching
        assert!(check("foo", "f.o")); // regex with any character

        assert!(!check("foo", "fo+oo")); // regex with at least one, but needs 0 repetitions
        assert!(!check("fOo", "foo")); // case mismatch, text upper-case
        assert!(!check("foo", "fOo")); // case mismatch, regex upper-case
        assert!(!check("föo", "foo")); // non-ASCII text, mismatch
        assert!(!check("foo", "föo")); // non-ASCII regex, mismatch
        assert!(!check("föo", "fÖo")); // non-ASCII, regex upper-case
        assert!(!check("fÖo", "föo")); // non-ASCII, text upper-case
    }

    #[test]
    fn insensitive() {
        fn check(text: &str, re: &str) -> bool {
            RegexMatcher::new(re.to_string())
                .get_matcher_func_case_insensitive()
                .unwrap()(&vkstr(text))
        }

        assert!(check("foo", "foo")); // full match
        assert!(check("foo", "oo")); // unanchored start
        assert!(check("foo", "fo")); // unanchored end
        assert!(check("foo", "fo*x*o")); // regex with arbirary repetitions
        assert!(check("foo", "fo+o")); // regex with at least one
        assert!(check("foo", "fo?oo")); // regex with optional
        assert!(check("foo", "^foo")); // regex with start matching
        assert!(check("foo", "foo$")); // regex with end matching
        assert!(check("foo", "f.o")); // regex with any character
        assert!(check("fOo", "foo")); // case mismatch, text upper-case
        assert!(check("foo", "fOo")); // case mismatch, regex upper-case
        assert!(check("föo", "fÖo")); // non-ASCII, regex upper-case
        assert!(check("fÖo", "föo")); // non-ASCII, text upper-case

        assert!(!check("foo", "fo+oo")); // regex with at least one, but needs 0 repetitions
        assert!(!check("föo", "foo")); // non-ASCII text, mismatch
        assert!(!check("foo", "föo")); // non-ASCII regex, mismatch
    }
}
