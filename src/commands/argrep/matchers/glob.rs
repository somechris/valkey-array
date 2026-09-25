use crate::commands::argrep::matchers::utils::ascii_lower_case;
use crate::commands::argrep::matchers::{Matcher, MatchingFn};
use valkey_module::{ValkeyResult, ValkeyString};

/// [`Matcher`] that matches only iff the candidate matches the gives glob
pub struct GlobMatcher {
    search_expr: ValkeyString,
}

impl GlobMatcher {
    /// Builds a new matcher for the given search expression
    pub fn new(search_expr: ValkeyString) -> Self {
        Self { search_expr }
    }
}
impl Matcher for GlobMatcher {
    fn get_matcher_func_case_sensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = self.search_expr;
        Ok(Box::new(move |candidate| {
            fast_glob::glob_match(&*search_expr, &**candidate)
        }))
    }

    fn get_matcher_func_case_insensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = ascii_lower_case(&self.search_expr);
        Ok(Box::new(move |candidate| {
            fast_glob::glob_match(&search_expr, ascii_lower_case(candidate))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{GlobMatcher, Matcher};
    use crate::utils::test_utils::vkstr;

    #[test]
    fn sensitive() {
        fn check(text: &str, glob: &str) -> bool {
            GlobMatcher::new(vkstr(glob))
                .get_matcher_func_case_sensitive()
                .unwrap()(&vkstr(text))
        }

        assert!(check("foo", "foo")); // Full match
        assert!(check("foo", "*o")); // Wildcard start
        assert!(check("foo", "f*")); // Wildcard end
        assert!(check("foo", "*f*")); // Wildcard anchoring
        assert!(check("fOO", "f[A-Z]O*")); // Mixed case
        assert!(check("föo", "föo")); // non-ASCII exact
        assert!(check("föo", "f*o")); // non-ASCII wildcard

        assert!(!check("foo", "o")); // Only middle, no start/end match
        assert!(!check("foo", "fo")); // No end match
        assert!(!check("foo", "oo")); // No start match
        assert!(!check("foo", "*oO")); // case mismatch, glob upper case
        assert!(!check("foO", "*oo")); // case mismatch, text upper case
        assert!(!check("föo", "foo")); // non-ASCII text mismatch
        assert!(!check("foo", "föo")); // non-ASCII glob mismatch
        assert!(!check("föo", "fÖ*")); // non-ASCII case mismatch, glob upper case
        assert!(!check("fÖo", "fö*")); // non-ASCII case mismatch, text upper case
    }

    #[test]
    fn insensitive() {
        fn check(text: &str, glob: &str) -> bool {
            GlobMatcher::new(vkstr(glob))
                .get_matcher_func_case_insensitive()
                .unwrap()(&vkstr(text))
        }

        assert!(check("foo", "foo")); // Full match
        assert!(check("foo", "*o")); // Wildcard start
        assert!(check("foo", "f*")); // Wildcard end
        assert!(check("foo", "*f*")); // Wildcard anchoring
        assert!(check("fOO", "f[A-Z]O*")); // Mixed case
        assert!(check("föo", "föo")); // non-ASCII exact
        assert!(check("föo", "f*o")); // non-ASCII wildcard
        assert!(check("foo", "*oO")); // case mismatch, glob upper case
        assert!(check("foO", "*oo")); // case mismatch, text upper case

        assert!(!check("foo", "o")); // Only middle, no start/end match
        assert!(!check("foo", "fo")); // No end match
        assert!(!check("foo", "oo")); // No start match
        assert!(!check("föo", "foo")); // non-ASCII text mismatch
        assert!(!check("foo", "föo")); // non-ASCII glob mismatch
        assert!(!check("föo", "fÖ*")); // non-ASCII case mismatch, glob upper case
        assert!(!check("fÖo", "fö*")); // non-ASCII case mismatch, text upper case
    }
}
