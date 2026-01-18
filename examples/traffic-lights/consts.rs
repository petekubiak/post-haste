use tokio::time::Duration;

pub const AMBER_TO_GREEN_DELAY: Duration = Duration::from_secs(3);
pub const GREEN_TO_AMBER_DELAY: Duration = Duration::from_secs(3);
pub const AMBER_TO_RED_DELAY: Duration = Duration::from_secs(3);
pub const CROSSING_START_DELAY: Duration = Duration::from_secs(4);
pub const CROSSING_LENGTH: Duration = Duration::from_secs(5);
pub const CROSSING_END_DELAY: Duration = Duration::from_secs(4);

pub const MAX_MESSAGES: usize = 3;
