pub const SECTION: &str = "partsneeded";

pub fn check<T>(what: &str, unit: i32, result: spring::Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            spring::log_warning!(SECTION, "{what} failed for unit {unit}: {error:?}");
            None
        }
    }
}
