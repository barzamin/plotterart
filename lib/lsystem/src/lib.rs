//! a basic implementation of Lindenmayer Systems for Rust.
//! probably not particularly well-optimized, but made with <3

use core::hash::Hash;
use std::collections::HashMap;

// currently, can only represent non-stochastic, context-free L-systems
type ProductionRules<Symbol> = HashMap<Symbol, Vec<Symbol>>;

pub struct ParametricLSystem<Symbol>
where
    Symbol: Eq + Hash + Clone,
{
    pub rules: ProductionRules<Symbol>,
    pub state: Vec<Symbol>,
}

impl<Symbol> ParametricLSystem<Symbol>
where
    Symbol: Eq + Hash + Clone,
{
    pub fn new(axiom: Vec<Symbol>, rules: ProductionRules<Symbol>) -> Self {
        Self {
            rules,
            state: axiom,
        }
    }

    pub fn evolve(&self) -> Vec<Symbol> {
        let mut expanded = Vec::with_capacity(self.state.len());
        for symbol in self.state.iter() {
            if let Some(replacement) = self.rules.get(symbol) {
                expanded.extend(replacement.iter().cloned());
            } else {
                expanded.push(symbol.clone());
            }
        }

        expanded
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn algae() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        enum Symbol {
            A,
            B,
        }
        use Symbol::*;

        let mut system = ParametricLSystem::new(
            vec![Symbol::A],
            hashmap! {
                A => vec![A, B],
                B => vec![A],
            },
        );

        assert_eq!(system.state, &[A]);
        system.state = system.evolve();
        assert_eq!(system.state, &[A, B]);
        system.state = system.evolve();
        assert_eq!(system.state, &[A, B, A]);
        system.state = system.evolve();
        assert_eq!(system.state, &[A, B, A, A, B]);
        system.state = system.evolve();
        assert_eq!(system.state, &[A, B, A, A, B, A, B, A]);
        system.state = system.evolve();
        assert_eq!(system.state, &[A, B, A, A, B, A, B, A, A, B, A, A, B]);
    }
}
