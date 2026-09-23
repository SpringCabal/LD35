//! Reporting engine callouts that failed.

use alloc::format;

/// Log a failed callout on `unit` instead of silently dropping it.
pub fn check<T>(what: &str, unit: i32, result: spring::Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            spring_addons::log::warning(&format!("{what} failed for unit {unit}: {error:?}"));
            None
        }
    }
}
