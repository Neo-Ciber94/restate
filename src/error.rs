use std::fmt::{Debug, Display};

/// An error ocurred during a transition.
pub enum TransitionError {
    // If the state machine is done.
    Done,

    // If the transition is not defined.
    InvalidTransition,
}

impl std::error::Error for TransitionError {}

impl Debug for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Done => write!(f, "state machine is done"),
            Self::InvalidTransition => write!(f, "invalid transition"),
        }
    }
}

impl Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}
