/// An immutable context.
#[derive(Debug)]
pub struct Context<'a, S, E, Ctx> {
    /// The state where this transition starts.
    pub from: &'a S,

    /// The state where this transition ends.
    pub to: &'a S,

    /// The event that triggers this transition.
    pub event: &'a E,

    /// The data associated to the state machine.
    pub context: &'a Ctx,
}

/// A mutable context.
#[derive(Debug)]
pub struct ContextMut<'a, S, E, Ctx> {
    /// The state where this transition starts.
    pub from: &'a S,

    /// The state where this transition ends.
    pub to: &'a S,

    /// The event that triggers this transition.
    pub event: &'a E,

    /// The mutable data associated to the state machine.
    pub context: &'a mut Ctx,
}
