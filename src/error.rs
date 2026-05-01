#[cfg(target_os = "none")]
pub mod imports {
    pub use embassy_executor::SpawnError;
    pub use embassy_sync::{channel::TrySendError, mutex::TryLockError};
    pub use embassy_time::TimeoutError;
}
#[cfg(not(target_os = "none"))]
pub mod imports {
    pub use tokio::sync::{TryLockError, mpsc::error::SendError, mpsc::error::TrySendError};
}

use imports::*;

/// Enumeration of potential errors which the Postmaster may encounter
#[derive(Debug)]
pub enum PostmasterError {
    /// The address specified has already been assigned
    AddressAlreadyTaken,
    /// No recipient has been registered at the specified address
    NoRecipient,
    /// The timeout was triggered while attempting to send a message
    Timeout,
    /// Postmaster was unable to acquire a lock on the Senders when `postmaster::try_send()` was called.
    /// This may happen if another task is waiting to send a message at the same time.
    TryLockFailed,
    /// The Receiver for the specified address has closed (gone out of scope).
    #[cfg(not(target_os = "none"))]
    ReceiverClosed, // Tokio Specific
    /// Calling `try_send()` on the recipient's message queue failed.
    /// This is most likely due to teh recipient's message queue being full.
    TrySendFailed,
    /// Postmaster was unable to spawn a task to handle the delayed message.
    /// This is most likely caused by the task pool being too small.
    /// Try increasing the DELAYED_MESSAGE_POOL_SIZE environment variable (default is 8).
    #[cfg(target_os = "none")]
    DelayedMessagePoolFull,
    /// A reference to the spawner has not yet been passed to the Postmaster.
    /// This is usually achieved automatically when `register_agent!()` is called.
    /// If you have not yet registered any Agents, you can call `postmaster::set_spawner()` before attempting to send the delayed message.
    #[cfg(target_os = "none")]
    SpawnerNotSet,
    /// Embassy failed to spawn a task. Currently this can only happen due to having
    /// too many instances of that task already running. This would indicate a bug
    /// in the post-haste source code.
    #[cfg(target_os = "none")]
    SpawnFailed,
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

#[cfg(target_os = "none")]
impl From<TimeoutError> for PostmasterError {
    fn from(_: TimeoutError) -> Self {
        Self::Timeout
    }
}

#[cfg(target_os = "none")]
impl From<SpawnError> for PostmasterError {
    fn from(_: SpawnError) -> Self {
        Self::DelayedMessagePoolFull
    }
}

#[cfg(not(target_os = "none"))]
impl<T> From<SendError<T>> for PostmasterError {
    fn from(_: SendError<T>) -> Self {
        Self::ReceiverClosed
    }
}
