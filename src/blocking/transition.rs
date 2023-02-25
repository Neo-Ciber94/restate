use crate::blocking::OnAction;
use private::*;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Represents a transition from an state to other state when an event arrives.
pub struct Transition<'a, S, E, Ctx> {
    pub(crate) from: S,
    pub(crate) to: S,
    pub(crate) event: E,
    pub(crate) is_final: bool,
    pub(crate) action: Option<Box<dyn OnAction<S, E, Ctx> + Send + 'a>>,
}

impl<S, E, Ctx> Debug for Transition<'_, S, E, Ctx>
where
    S: Debug,
    E: Debug,
    Ctx: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transition")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("event", &self.event)
            .field("is_final", &self.is_final)
            .field("action", {
                match self.action {
                    None => &"None",
                    Some(_) => &"Some(TransitionAction)",
                }
            })
            .finish()
    }
}

/// Allows a type to be converted into a `Transition`.
pub trait IntoTransition<'a, S, E, Ctx> {
    /// Converts this type into a `Transition`.
    fn into_transition(self) -> Transition<'a, S, E, Ctx>;
}

/// A `Transition` builder.
pub struct Builder<'a, S, E, Ctx, TStep = Build> {
    from: Option<S>,
    to: Option<S>,
    event: E,
    is_final: bool,
    action: Option<Box<dyn OnAction<S, E, Ctx> + Send + 'a>>,
    _marker: PhantomData<TStep>,
}

impl<'a, S, E, Ctx> Builder<'a, S, E, Ctx, Build> {
    /// Constructs a transition that goes from and start to end state when the given event is emitted.
    pub fn new(from: S, to: S, event: E) -> Builder<'a, S, E, Ctx, CanBuild>
    where
        S: Clone,
    {
        Builder {
            from: Some(from),
            to: Some(to),
            event,
            is_final: false,
            action: None,
            _marker: PhantomData,
        }
    }

    /// Trigger the transition when the given event happens.
    pub fn when(event: E) -> Self {
        Builder {
            from: None,
            to: None,
            event,
            is_final: false,
            action: None,
            _marker: PhantomData,
        }
    }

    /// Trigger a transition from and state to itself when the given event happens.
    pub fn self_transition(event: E, state: S) -> Builder<'a, S, E, Ctx, CanBuild>
    where
        S: Clone,
    {
        Builder::new(state.clone(), state, event)
    }

    /// Sets the state from which the transition starts.
    pub fn from(self, state: S) -> Builder<'a, S, E, Ctx, HasFrom> {
        Builder {
            from: Some(state),
            to: None,
            event: self.event,
            is_final: self.is_final,
            action: self.action,
            _marker: PhantomData,
        }
    }
}

impl<'a, S, E, Ctx> Builder<'a, S, E, Ctx, HasFrom> {
    /// Sets the type where the transition goes to.
    pub fn to(self, state: S) -> Builder<'a, S, E, Ctx, CanBuild> {
        Builder {
            from: self.from,
            to: Some(state),
            event: self.event,
            is_final: self.is_final,
            action: self.action,
            _marker: PhantomData,
        }
    }
}

impl<'a, S, E, Ctx> Builder<'a, S, E, Ctx, CanBuild> {
    /// Ensure this transition completes the state machine.
    pub fn is_final(mut self) -> Self {
        self.is_final = true;
        self
    }

    /// Sets an action to execute this transition happen.
    pub fn action<F>(mut self, f: F) -> Self
    where
        F: OnAction<S, E, Ctx> + Send + 'a,
    {
        self.action = Some(Box::new(f));
        self
    }
}

impl<'a, S, E, Ctx> IntoTransition<'a, S, E, Ctx> for Builder<'a, S, E, Ctx, CanBuild> {
    fn into_transition(self) -> Transition<'a, S, E, Ctx> {
        Transition {
            event: self.event,
            from: self.from.unwrap(),
            to: self.to.unwrap(),
            action: self.action,
            is_final: self.is_final,
        }
    }
}

/// Returns a transition builder when the given events happen.
pub fn when<'a, S, E, Ctx>(event: E) -> Builder<'a, S, E, Ctx, Build> {
    Builder::when(event)
}

/// Returns a transition builder that make a transition to itself when the given events happen.
pub fn self_transition<'a, S, E, Ctx>(event: E, state: S) -> Builder<'a, S, E, Ctx, CanBuild>
where
    S: Clone,
{
    Builder::self_transition(event, state)
}

/// Zero types that represent the state of a transition `Builder`.
pub(crate) mod private {
    #[derive(Debug, Clone)]
    pub struct Build;

    #[derive(Debug, Clone)]
    pub struct Ready;

    #[derive(Debug, Clone)]
    pub struct HasFrom;

    #[derive(Debug, Clone)]
    pub struct HasTo;

    #[derive(Debug, Clone)]
    pub struct CanBuild;
}
