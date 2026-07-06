//! podbox library crate.
//!
//! The binary is a thin shell over this library so the logic stays unit-,
//! integration-, and doc-testable. This `greeting` is a placeholder standing in
//! until the real implementation lands.

/// Returns the crate's startup banner.
///
/// # Examples
///
/// ```
/// assert_eq!(podbox::greeting(), "Hello, world!");
/// ```
pub fn greeting() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod tests {
    use super::greeting;

    #[test]
    fn greeting_is_stable() {
        assert_eq!(greeting(), "Hello, world!");
    }
}
