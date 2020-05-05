use crate::parser::symbol::Symbol;
use derive_more::{From, Index};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Index, From)]
pub struct Production(pub Vec<Symbol>);

impl Production {
    pub fn contains_terminal(&self) -> bool {
        self.0.iter().any(|s| s.is_terminal())
    }

    pub fn index_of(&self, symbol: &Symbol) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .find(|(i, s)| *s == symbol)
            .map(|(i, s)| i)
    }

    pub fn only_lambda(&self) -> bool {
        self.0.len() == 1 && self.0[0] == Symbol::Lambda
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.symbols().len()
    }
}

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for symbol in self.0.iter() {
            write!(f, "{} ", symbol.as_str())?;
        }

        Ok(())
    }
}
