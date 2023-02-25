use super::ContextMut;

/// An action executed when state machine enters to a new state.
pub trait OnAction<S, E, Ctx> {
    /// Function called when entering a new state.
    fn call(&mut self, cx: ContextMut<S, E, Ctx>);
}

impl<S, E, Ctx, F> OnAction<S, E, Ctx> for F
where
    F: FnMut(ContextMut<S, E, Ctx>),
{
    fn call(&mut self, cx: ContextMut<S, E, Ctx>) {
        (self)(cx)
    }
}

impl<S, E, Ctx> OnAction<S, E, Ctx> for () {
    fn call(&mut self, _: ContextMut<S, E, Ctx>) {}
}
