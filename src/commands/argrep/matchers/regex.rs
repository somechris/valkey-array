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
    use crate::test_utils::vkstr;

    #[test]
    fn sensitive() {
        let expr = "b.*[rZ]";
        let match_fn = RegexMatcher::new(expr.to_string())
            .get_matcher_func_case_sensitive()
            .unwrap();

        assert!(match_fn(&vkstr("br"))); // no wildcard, r (lowercase)
        assert!(match_fn(&vkstr("bZ"))); // no wildcard, Z (uppercase)
        assert!(match_fn(&vkstr("bar"))); // simple match
        assert!(match_fn(&vkstr("  bar"))); // unanchored start
        assert!(match_fn(&vkstr("bar  "))); // unanchored end

        assert!(!match_fn(&vkstr("bR"))); // no wildcard, R (uppercase)
        assert!(!match_fn(&vkstr("bz"))); // no wildcard, z (lowercase)
        assert!(!match_fn(&vkstr("bak"))); // Missing [rz]
    }

    #[test]
    fn insensitive() {
        let expr = "b.*[rZ]";
        let match_fn = RegexMatcher::new(expr.to_string())
            .get_matcher_func_case_insensitive()
            .unwrap();

        assert!(match_fn(&vkstr("br"))); // no wildcard, r (lowercase)
        assert!(match_fn(&vkstr("bR"))); // no wildcard, R (uppercase)
        assert!(match_fn(&vkstr("bz"))); // no wildcard, z (lowercase)
        assert!(match_fn(&vkstr("bZ"))); // no wildcard, Z (uppercase)
        assert!(match_fn(&vkstr("bar"))); // simple match
        assert!(match_fn(&vkstr("  bar"))); // unanchored start
        assert!(match_fn(&vkstr("bar  "))); // unanchored end

        assert!(!match_fn(&vkstr("bak"))); // Missing [rZ]
    }
}
