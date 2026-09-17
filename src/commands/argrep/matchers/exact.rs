use crate::commands::argrep::matchers::{Matcher, MatchingFn};
use valkey_module::{ValkeyResult, ValkeyString};

/// [`Matcher`] that matches only iff the candidate and the search expression are the same
pub struct ExactMatcher {
    search_expr: ValkeyString,
}

impl ExactMatcher {
    /// Builds a new matcher for the given search expression
    pub fn new(search_expr: ValkeyString) -> Self {
        Self { search_expr }
    }
}

impl Matcher for ExactMatcher {
    fn get_matcher_func_case_sensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = self.search_expr;
        Ok(Box::new(move |candidate| candidate == &search_expr))
    }

    fn get_matcher_func_case_insensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = self.search_expr;
        Ok(Box::new(move |candidate| {
            (*candidate).eq_ignore_ascii_case(&search_expr)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{ExactMatcher, Matcher};
    use crate::test_utils::vkstr;

    #[test]
    fn sensitive() {
        let expr = "foo";
        let match_fn = ExactMatcher::new(vkstr(expr))
            .get_matcher_func_case_sensitive()
            .unwrap();

        assert!(match_fn(&vkstr("foo")));

        assert!(!match_fn(&vkstr("  foo"))); // Prefixed whitespace
        assert!(!match_fn(&vkstr("f oo"))); // Infixed whitespace
        assert!(!match_fn(&vkstr("foo  "))); // Postfixed whitespace
        assert!(!match_fn(&vkstr("foO"))); // different case
        assert!(!match_fn(&vkstr("barfoobaz"))); // Padded words
    }

    #[test]
    fn insensitive() {
        let expr = "foo";
        let match_fn = ExactMatcher::new(vkstr(expr))
            .get_matcher_func_case_insensitive()
            .unwrap();

        assert!(match_fn(&vkstr("foo")));
        assert!(match_fn(&vkstr("fOo")));
        assert!(match_fn(&vkstr("FOO")));

        assert!(!match_fn(&vkstr("  foo"))); // Prefixed whitespace
        assert!(!match_fn(&vkstr("f oo"))); // Infixed whitespace
        assert!(!match_fn(&vkstr("foo  "))); // Postfixed whitespace
        assert!(!match_fn(&vkstr("barfoobaz"))); // Padded words
    }
}
