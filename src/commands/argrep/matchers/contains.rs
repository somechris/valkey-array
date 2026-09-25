use crate::commands::argrep::matchers::utils::ascii_lower_case;
use crate::commands::argrep::matchers::{Matcher, MatchingFn};
use valkey_module::{ValkeyResult, ValkeyString};

/// [`Matcher`] that matches only iff the candidate contains the search expression
pub struct ContainsMatcher {
    search_expr: ValkeyString,
}

impl ContainsMatcher {
    /// Builds a new matcher for the given search expression
    pub fn new(search_expr: ValkeyString) -> Self {
        Self { search_expr }
    }
}

impl Matcher for ContainsMatcher {
    fn get_matcher_func_case_sensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = self.search_expr;

        Ok(Box::new(move |candidate| {
            candidate
                .windows(search_expr.len())
                .any(|left| left == &*search_expr)
        }))
    }

    fn get_matcher_func_case_insensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr_lc = ascii_lower_case(&self.search_expr);
        Ok(Box::new(move |candidate| {
            ascii_lower_case(candidate)
                .windows(search_expr_lc.len())
                .any(|left| left == search_expr_lc)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{ContainsMatcher, Matcher};
    use crate::utils::test_utils::vkstr;

    #[test]
    fn sensitive() {
        let expr = "foo";
        let match_fn = ContainsMatcher::new(vkstr(expr))
            .get_matcher_func_case_sensitive()
            .unwrap();

        assert!(match_fn(&vkstr("foo"))); // Exact match
        assert!(match_fn(&vkstr("  foo"))); // Prefixed whitespace
        assert!(match_fn(&vkstr("foo  "))); // Postfixed whitespace
        assert!(match_fn(&vkstr("barfoobaz"))); // Padded words

        assert!(!match_fn(&vkstr("f oo"))); // Infixed whitespace
        assert!(!match_fn(&vkstr("foO"))); // different case
    }

    #[test]
    fn insensitive() {
        let expr = "foo";
        let match_fn = ContainsMatcher::new(vkstr(expr))
            .get_matcher_func_case_insensitive()
            .unwrap();

        assert!(match_fn(&vkstr("foo"))); // Exact match
        assert!(match_fn(&vkstr("fOo"))); // Partially different case
        assert!(match_fn(&vkstr("FOO"))); // Fully different case
        assert!(match_fn(&vkstr("  foo"))); // Prefixed whitespace
        assert!(match_fn(&vkstr("foo  "))); // Postfixed whitespace
        assert!(match_fn(&vkstr("barfoobaz"))); // Padded words
        assert!(match_fn(&vkstr("barFOobaz "))); // Mix

        assert!(!match_fn(&vkstr("f oo"))); // Infixed whitespace
        assert!(!match_fn(&vkstr("bar"))); // Different word
    }
}
