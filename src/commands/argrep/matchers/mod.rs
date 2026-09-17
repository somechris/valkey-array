//! Matcher implementations for `ARGREP`

use valkey_module::{ValkeyResult, ValkeyString};

mod contains;
mod exact;
mod glob;
mod regex;

mod utils;

pub use contains::ContainsMatcher;
pub use exact::ExactMatcher;
pub use glob::GlobMatcher;
pub use regex::RegexMatcher;

/// Function that matches an item
pub type MatchingFn = Box<dyn Fn(&ValkeyString) -> bool>;

/// Trait to model a matching operator
pub trait Matcher {
    /// Gets the matching function for case-sensitive matching
    fn get_matcher_func_case_sensitive(self) -> ValkeyResult<MatchingFn>;

    /// Gets the matching function for case-insensitive matching
    fn get_matcher_func_case_insensitive(self) -> ValkeyResult<MatchingFn>;

    /// Gets one of the matcher functions
    fn get_matcher_func(self, case_sensitive: bool) -> ValkeyResult<MatchingFn>
    where
        Self: Sized,
    {
        if case_sensitive {
            self.get_matcher_func_case_sensitive()
        } else {
            self.get_matcher_func_case_insensitive()
        }
    }
}
