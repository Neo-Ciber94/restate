#![allow(dead_code)]

use std::slice::Iter;

#[derive(Debug, Clone)]
struct To<TEvent, T> {
    event: TEvent,
    to: T,
}

#[derive(Debug, Clone)]
struct Node<TState, TEvent, T> {
    from: TState,
    next: Vec<To<TEvent, T>>,
}

// This corrent implementation of the `TransitionMap` is O(n) in most of the operations.

/// A map that store the states and its transitions to other states when a event happens.
#[derive(Debug, Clone)]
pub struct TransitionMap<TState, TEvent, T> {
    nodes: Vec<Node<TState, TEvent, T>>,
}

impl<TState, TEvent, T> TransitionMap<TState, TEvent, T> {
    pub fn new() -> Self {
        TransitionMap { nodes: Vec::new() }
    }

    pub fn events(&self) -> Events<'_, TState, TEvent, T> {
        Events {
            iter: self.nodes.iter(),
            cur: None,
        }
    }

    pub fn states(&self) -> States<'_, TState, TEvent, T> {
        States {
            iter: self.nodes.iter(),
        }
    }
}

impl<TState, TEvent, T> TransitionMap<TState, TEvent, T>
where
    TState: PartialEq,
    TEvent: PartialEq,
{
    pub fn insert(&mut self, event: TEvent, from: TState, to: T) {
        let next = self
            .nodes
            .iter_mut()
            .find(|node| node.from == from)
            .map(|x| &mut x.next);

        match next {
            Some(next) => {
                // We can only trigger 1 transition per event,
                // so if the transition already exists for that event we panic
                let exists = next.iter().any(|x| x.event == event);
                if exists {
                    panic!("a transition already exists for the event");
                }

                next.push(To { event, to });
            }
            None => {
                // Insert node
                self.nodes.push(Node {
                    from,
                    next: vec![To { event, to }],
                });
            }
        }
    }

    pub fn get(&self, event: &TEvent, from: &TState) -> Option<&T> {
        self.nodes
            .iter()
            .filter(|node| &node.from == from)
            .flat_map(|node| node.next.iter())
            .find(|next| &next.event == event)
            .map(|next| &next.to)
    }

    pub fn get_mut(&mut self, event: &TEvent, from: &TState) -> Option<&mut T> {
        self.nodes
            .iter_mut()
            .filter(|node| &node.from == from)
            .flat_map(|node| node.next.iter_mut())
            .find(|next| &next.event == event)
            .map(|next| &mut next.to)
    }
}

/// An iterator over the states.
#[derive(Debug, Clone)]
pub struct States<'a, S, E, T> {
    iter: Iter<'a, Node<S, E, T>>,
}

impl<'a, S, E, T> Iterator for States<'a, S, E, T> {
    type Item = &'a S;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|node| &node.from)
    }
}

/// An iterator over the events.
#[derive(Debug, Clone)]
pub struct Events<'a, S, E, T> {
    iter: Iter<'a, Node<S, E, T>>,
    cur: Option<Iter<'a, To<E, T>>>,
}

impl<'a, S, E, T> Iterator for Events<'a, S, E, T> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.cur.as_mut().and_then(|x| x.next()) {
            return Some(&cur.event);
        }

        match self.iter.next() {
            Some(node) => {
                self.cur = Some(node.next.iter());
                self.next()
            }
            None => None,
        }
    }
}
