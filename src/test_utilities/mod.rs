use std::time::{Duration, SystemTime};

/// Returns a timestamp elasped `milis` milliseconds from UNIX EPOCH.
/// For testing purposes only.
pub fn mipoch(milis: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(milis)
}
