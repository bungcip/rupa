//! Shared helpers for compiler unit tests.

use rupa_syntax::diagnostic::DiagnosticBag;

/// Collected diagnostic codes in emission order.
pub(crate) fn codes_of(bag: &DiagnosticBag) -> Vec<&str> {
    bag.iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}
