use tokio::time::Duration;

// Some constants defining the length of each section of the sequence
pub const AMBER_TO_GREEN_DELAY: Duration = Duration::from_secs(2);
pub const GREEN_TO_AMBER_DELAY: Duration = Duration::from_secs(2);
pub const AMBER_TO_RED_DELAY: Duration = Duration::from_secs(2);
pub const CROSSING_START_DELAY: Duration = Duration::from_secs(3);
pub const CROSSING_LENGTH: Duration = Duration::from_secs(3);
pub const CROSSING_END_DELAY: Duration = Duration::from_secs(2);

// The maximum number of debug messages for the display agent
pub const MAXIMUM_DEBUG_MESSAGES: usize = 10;
