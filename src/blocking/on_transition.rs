use super::Context;

/// An action executed when a transition happens.
pub trait OnTransition<S, E, Ctx> {
    /// Function called when a transition occurred.
    fn call(&mut self, cx: Context<S, E, Ctx>);
}

impl<S, E, Ctx, F> OnTransition<S, E, Ctx> for F
where
    F: FnMut(Context<S, E, Ctx>),
{
    fn call(&mut self, cx: Context<S, E, Ctx>) {
        (self)(cx)
    }
}

impl<S, E, Ctx> OnTransition<S, E, Ctx> for () {
    fn call(&mut self, _cx: Context<S, E, Ctx>) {}
}
