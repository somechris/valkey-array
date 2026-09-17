//! Utilities to implement matchers

/// Converts the input's uppercase ASCII characters to lowercase
pub fn ascii_lower_case(input: &[u8]) -> Vec<u8> {
    input.iter().map(u8::to_ascii_lowercase).collect()
}
