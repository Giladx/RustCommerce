use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderTransitionError {
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: String,
        to: String,
    },

    #[error("order already finalized")]
    AlreadyFinalized,
}
