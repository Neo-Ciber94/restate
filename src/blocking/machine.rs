use super::{Context, ContextMut, OnAction};
use crate::blocking::OnTransition;
use crate::blocking::{IntoTransition, Transition};
use crate::common::map::{Events, States, TransitionMap};
use crate::error::TransitionError;
pub use private::*;
use std::{fmt::Debug, marker::PhantomData};

#[doc(hidden)]
pub struct Next<'a, S, E, Ctx> {
    next: S,
    is_final: bool,
    action: Option<Box<dyn OnAction<S, E, Ctx> + Send + 'a>>,
}

impl<S, E, Ctx> Debug for Next<'_, S, E, Ctx>
where
    S: Debug,
    Ctx: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("next", &self.next)
            .field("is_final", &self.is_final)
            .finish()
    }
}

/// Represents a finite state machine that can transition between different states based on events.
///
/// # Example
///
/// ```rust
/// use restate::blocking::*;
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct Active;
///
/// #[derive(PartialEq, Eq)]
/// enum CountEvent {
///     Increment,
///     Decrement,
/// }
///
/// let mut sm = Machine::with_context(0)
///     .on_next(
///         Builder::self_transition(Active, CountEvent::Increment).action(
///             |cx: ContextMut<Active, CountEvent, i32>| {
///                 *cx.context += 1;
///             },
///         ),
///     )
///     .on_next(
///         Builder::self_transition(Active, CountEvent::Decrement).action(
///             |cx: ContextMut<Active, CountEvent, i32>| {
///                 *cx.context -= 1;
///             },
///         ),
///     )
///     .start(Active);
///
/// sm.send(CountEvent::Increment).unwrap();
/// sm.send(CountEvent::Increment).unwrap();
/// sm.send(CountEvent::Increment).unwrap();
/// sm.send(CountEvent::Decrement).unwrap();
///
/// assert_eq!(*sm.context(), 2);
/// ```
pub struct Machine<'a, S, E, Ctx, F, Step = Build> {
    // A map of state and event transitions to the next state and associated action.
    transitions: TransitionMap<S, E, Next<'a, S, E, Ctx>>,

    // The current state of the machine, will be `None` if the machine had not started.
    current: Option<S>,

    // Indicates whether the state machine has finished execution.
    done: bool,

    // A context object for storing and passing data between state transitions.
    context: Ctx,

    // An optional callback function to execute when a transition occurs.
    on_transition: Option<F>,

    _marker: PhantomData<Step>,
}

impl<S, E, Ctx, F, Step> Debug for Machine<'_, S, E, Ctx, F, Step>
where
    S: Debug,
    E: Debug,
    Ctx: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("current", &self.current)
            .field("done", &self.done)
            .field("context", &self.context)
            .field("transitions", &self.transitions)
            .finish()
    }
}

impl<'a, S, E> Machine<'a, S, E, (), (), Build> {
    /// Returns a new `StateMachine`.
    pub fn new() -> Machine<'a, S, E, (), (), Build> {
        Machine {
            transitions: TransitionMap::new(),
            current: None,
            done: false,
            context: (),
            on_transition: None,
            _marker: PhantomData,
        }
    }

    /// Returns a new `StateMachine` with the given context.
    pub fn with_context<Ctx>(context: Ctx) -> Machine<'a, S, E, Ctx, (), Build> {
        Machine {
            transitions: TransitionMap::new(),
            current: None,
            done: false,
            context,
            on_transition: None,
            _marker: PhantomData,
        }
    }
}

impl<'a, S, E, Ctx> Machine<'a, S, E, Ctx, (), Build>
where
    E: PartialEq,
    S: PartialEq,
{
    /// Adds a transition from a state to other based on an event.
    pub fn on_next(mut self, transition: impl IntoTransition<'a, S, E, Ctx>) -> Self {
        let Transition {
            from,
            to,
            event,
            action,
            is_final,
        } = transition.into_transition();

        self.transitions.insert(
            event,
            from,
            Next {
                next: to,
                action,
                is_final,
            },
        );
        self
    }

    /// Adds a function that is called when a transition occurs.
    pub fn on_transition<F>(self, on_transition: F) -> Machine<'a, S, E, Ctx, F, Build>
    where
        F: FnMut(Context<S, E, Ctx>),
    {
        Machine {
            current: self.current,
            transitions: self.transitions,
            done: false,
            context: self.context,
            on_transition: Some(on_transition),
            _marker: PhantomData,
        }
    }
}

impl<'a, S, E, F, Ctx> Machine<'a, S, E, Ctx, F, Build> {
    /// Starts this state machine with the given state.
    pub fn start(self, initial_state: S) -> Machine<'a, S, E, Ctx, F, Ready> {
        Machine {
            current: Some(initial_state),
            transitions: self.transitions,
            done: false,
            context: self.context,
            on_transition: self.on_transition,
            _marker: PhantomData,
        }
    }
}

impl<S, E, F, Ctx> Machine<'_, S, E, Ctx, F, Ready>
where
    E: PartialEq,
    S: PartialEq + Clone,
    F: OnTransition<S, E, Ctx>,
{
    /// Returns the states of the state machine.
    pub fn states(&self) -> States<'_, S, E, Next<S, E, Ctx>> {
        self.transitions.states()
    }

    /// Returns the events of the state machine.
    pub fn events(&self) -> Events<'_, S, E, Next<S, E, Ctx>> {
        self.transitions.events()
    }

    /// Returns the current state.
    pub fn current(&self) -> &S {
        self.current.as_ref().unwrap()
    }

    /// Returns the context used for this state machine.
    pub fn context(&self) -> &Ctx {
        &self.context
    }

    /// Returns `true` if this state machine had done executing.
    pub fn is_done(&self) -> bool {
        self.done
    }
}
impl<S, E, F, Ctx> Machine<'_, S, E, Ctx, F, Ready>
where
    E: PartialEq,
    S: PartialEq + Clone,
    F: OnTransition<S, E, Ctx>,
{
    /// Triggers a transition.
    ///
    /// # Returns
    /// - Ok(S): The previous state.
    /// - Err(TransitionError): If the transition was not successful
    pub fn send(&mut self, event: E) -> Result<S, TransitionError> {
        if self.done {
            return Err(TransitionError::Done);
        }

        // SAFETY: If this state machine is in step `Ready`,
        // current state cannot be null
        let state = self.current.as_mut().unwrap();

        let Some(Next {
            next,
            action,
            is_final,
        }) = self.transitions.get_mut(&event, state) else {
            return Err(TransitionError::InvalidTransition);
        };

        // Set the new state
        // TODO: Check if this `Clone` can be removed
        let prev_state = std::mem::replace(state, next.clone());

        if *is_final {
            self.done = true;
        }

        // Call the action of the transition if any
        if let Some(f) = action.as_mut() {
            f.call(ContextMut {
                from: state,
                to: next,
                event: &event,
                context: &mut self.context,
            });
        }

        // After the transition is done, call the `on_transition`
        if let Some(f) = self.on_transition.as_mut() {
            f.call(Context {
                from: state,
                to: next,
                event: &event,
                context: &self.context,
            });
        }

        Ok(prev_state)
    }
}

impl<S, E> Default for Machine<'_, S, E, (), (), Build> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) mod private {
    #[derive(Debug, Clone)]
    pub struct Build;

    #[derive(Debug, Clone)]
    pub struct Ready;
}

#[cfg(test)]
mod tests {
    use crate::blocking::{Builder, ContextMut, Machine};

    #[test]
    fn send_test() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum LightState {
            On,
            Off,
        }

        #[derive(PartialEq, Eq)]
        enum LightEvent {
            TurnOn,
            TurnOff,
        }

        let mut sm = Machine::new()
            .on_next(
                Builder::new(LightState::Off)
                    .on(LightEvent::TurnOn)
                    .go_to(LightState::On),
            )
            .on_next(
                Builder::new(LightState::On)
                    .on(LightEvent::TurnOff)
                    .go_to(LightState::Off),
            )
            .start(LightState::Off);

        assert_eq!(sm.send(LightEvent::TurnOn).unwrap(), LightState::Off);
        assert_eq!(sm.send(LightEvent::TurnOff).unwrap(), LightState::On);
        assert!(sm.send(LightEvent::TurnOff).is_err());
    }

    #[test]
    fn on_transition_test() {
        let mut value = 0;

        let mut sm = Machine::new()
            .on_next(Builder::self_transition((), ()))
            .on_transition(|_| {
                value += 1;
            })
            .start(());

        sm.send(()).unwrap();
        sm.send(()).unwrap();

        assert_eq!(value, 2);
    }

    #[test]
    fn on_action_test() {
        let mut value = 0;

        {
            let mut sm = Machine::new()
                .on_next(
                    Builder::self_transition((), ()).action(|_: ContextMut<_, _, _>| {
                        value += 1;
                    }),
                )
                .start(());

            sm.send(()).unwrap();
            sm.send(()).unwrap();
        }

        assert_eq!(value, 2);
    }

    #[test]
    fn with_context_test() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Active;

        #[derive(PartialEq, Eq)]
        enum CountEvent {
            Increment,
            Decrement,
        }

        let mut sm = Machine::with_context(0)
            .on_next(
                Builder::self_transition(Active, CountEvent::Increment).action(
                    |cx: ContextMut<Active, CountEvent, i32>| {
                        *cx.context += 1;
                    },
                ),
            )
            .on_next(
                Builder::self_transition(Active, CountEvent::Decrement).action(
                    |cx: ContextMut<Active, CountEvent, i32>| {
                        *cx.context -= 1;
                    },
                ),
            )
            .start(Active);

        sm.send(CountEvent::Increment).unwrap();
        sm.send(CountEvent::Increment).unwrap();
        sm.send(CountEvent::Increment).unwrap();

        assert_eq!(*sm.context(), 3);

        sm.send(CountEvent::Decrement).unwrap();
        sm.send(CountEvent::Decrement).unwrap();

        assert_eq!(*sm.context(), 1);
    }
}
