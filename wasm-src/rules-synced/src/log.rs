pub const SECTION: &str = "partsneeded";

pub fn check<T>(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    what: &str,
    unit: i32,
    result: spring::Result<T>,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            spring::log_warning!(spring, SECTION, "{what} failed for unit {unit}: {error:?}");
            None
        }
    }
}
