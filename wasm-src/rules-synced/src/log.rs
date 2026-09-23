//! Reporting engine callouts that failed.

pub const SECTION: &str = "partsneeded";

/// Log a failed callout on `unit` instead of silently dropping it.
pub fn check<T>(what: &str, unit: i32, result: spring::Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            spring::log_warning!(SECTION, "{what} failed for unit {unit}: {error:?}");
            None
        }
    }
}
