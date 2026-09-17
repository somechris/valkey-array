use crate::commands::argrep::matchers::utils::ascii_lower_case;
use crate::commands::argrep::matchers::{Matcher, MatchingFn};
use valkey_module::{ValkeyResult, ValkeyString};

/// [`Matcher`] that matches only iff the candidate matches the gives glob
pub struct GlobMatcher {
    search_expr: Vec<u8>,
}

impl GlobMatcher {
    /// Builds a new matcher for the given search expression
    pub fn new(search_expr_raw: ValkeyString) -> Self {
        // Valkey checks globs as if they end in a `*`. `fast_glob` does not.
        // So we add a `*` it if necessary.
        let mut search_expr = search_expr_raw.to_vec();
        if !search_expr.ends_with("*".as_bytes()) {
            search_expr.push(0x2a); // Appending '*'
        }

        Self { search_expr }
    }
}
impl Matcher for GlobMatcher {
    fn get_matcher_func_case_sensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = self.search_expr;
        Ok(Box::new(move |candidate| {
            fast_glob::glob_match(&search_expr, &**candidate)
        }))
    }

    fn get_matcher_func_case_insensitive(self) -> ValkeyResult<MatchingFn> {
        let search_expr = self.search_expr;
        Ok(Box::new(move |candidate| {
            fast_glob::glob_match(&search_expr, ascii_lower_case(candidate))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{GlobMatcher, Matcher};
    use crate::test_utils::vkstr;

    #[test]
    fn sensitive_with_lowercase_glob() {
        let expr = "f*o";
        let match_fn = GlobMatcher::new(vkstr(expr))
            .get_matcher_func_case_sensitive()
            .unwrap();

        assert!(match_fn(&vkstr("fo"))); // `*` is empty
        assert!(match_fn(&vkstr("fao"))); // `*` is single character
        assert!(match_fn(&vkstr("f o"))); // `*` is whitespace
        assert!(match_fn(&vkstr("faBCdeo"))); // `*` is multiple characters
        assert!(match_fn(&vkstr("foo bar"))); // end is not anchored

        assert!(!match_fn(&vkstr(" foo"))); // start is anchored
        assert!(!match_fn(&vkstr("FO"))); // different case
        assert!(!match_fn(&vkstr("faa"))); // `o` is missing
        assert!(!match_fn(&vkstr("boo"))); // `f` is missing
    }

    #[test]
    fn sensitive_with_mixed_glob() {
        let expr = "f*O";
        let match_fn = GlobMatcher::new(vkstr(expr))
            .get_matcher_func_case_sensitive()
            .unwrap();

        assert!(match_fn(&vkstr("fO"))); // `*` is empty
        assert!(match_fn(&vkstr("faO"))); // `*` is single character
        assert!(match_fn(&vkstr("f O"))); // `*` is whitespace
        assert!(match_fn(&vkstr("faBCdeO"))); // `*` is multiple characters
        assert!(match_fn(&vkstr("foO bar"))); // end is not anchored

        assert!(!match_fn(&vkstr(" foO"))); // start is anchored
        assert!(!match_fn(&vkstr("FO"))); // different case
        assert!(!match_fn(&vkstr("foo"))); // different case
        assert!(!match_fn(&vkstr("faa"))); // `o` is missing
        assert!(!match_fn(&vkstr("boo"))); // `f` is missing
    }

    #[test]
    fn insensitive_with_lowercase_glob() {
        let expr = "f*o";
        let match_fn = GlobMatcher::new(vkstr(expr))
            .get_matcher_func_case_insensitive()
            .unwrap();

        assert!(match_fn(&vkstr("fo"))); // `*` is empty
        assert!(match_fn(&vkstr("fao"))); // `*` is single character
        assert!(match_fn(&vkstr("f o"))); // `*` is whitespace
        assert!(match_fn(&vkstr("faBCdeo"))); // `*` is multiple characters
        assert!(match_fn(&vkstr("foo bar"))); // end is not anchored
        assert!(match_fn(&vkstr("FO"))); // different case

        assert!(!match_fn(&vkstr(" foo"))); // start is anchored
        assert!(!match_fn(&vkstr("faa"))); // `o` is missing
        assert!(!match_fn(&vkstr("boo"))); // `f` is missing
    }

    #[test]
    fn insensitive_with_mixed_glob() {
        let expr = "f*O";
        let match_fn = GlobMatcher::new(vkstr(expr))
            .get_matcher_func_case_insensitive()
            .unwrap();

        assert!(!match_fn(&vkstr("fabco"))); // all lowercase
        assert!(!match_fn(&vkstr("fabcO"))); // lowercase and uppercase
        assert!(!match_fn(&vkstr("Fabco"))); // uppercase and lowercase
        assert!(!match_fn(&vkstr("FabcO"))); // all uppercase
        assert!(!match_fn(&vkstr("fao"))); // `*` is single character
        assert!(!match_fn(&vkstr("f o"))); // `*` is whitespace
        assert!(!match_fn(&vkstr("faBCdeo"))); // `*` is multiple characters
        assert!(!match_fn(&vkstr("foo bar"))); // end is not anchored
        assert!(!match_fn(&vkstr("FO"))); // different case

        assert!(!match_fn(&vkstr(" foo"))); // start is anchored
        assert!(!match_fn(&vkstr("faa"))); // `o` is missing
        assert!(!match_fn(&vkstr("boo"))); // `f` is missing
    }
}
