use embassy_executor::SpawnError;
use embassy_sync::{channel::TrySendError, mutex::TryLockError};
use embassy_time::TimeoutError;

#[derive(Debug)]
pub enum PostmasterError {
    AddressAlreadyTaken,
    NoRecipient,
    Timeout,
    TryLockFailed,
    TrySendFailed,
    DelayedMessageTaskSpawnFailed,
}

impl From<TimeoutError> for PostmasterError {
    fn from(_: TimeoutError) -> Self {
        Self::Timeout
    }
}

impl From<TryLockError> for PostmasterError {
    fn from(_: TryLockError) -> Self {
        Self::TryLockFailed
    }
}

impl<T> From<TrySendError<T>> for PostmasterError {
    fn from(_: TrySendError<T>) -> Self {
        Self::TrySendFailed
    }
}

impl From<SpawnError> for PostmasterError {
    fn from(_: SpawnError) -> Self {
        Self::DelayedMessageTaskSpawnFailed
    }
}
