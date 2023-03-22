pub use pest::iterators::*;
pub use pest::Parser;

use std::fmt;
use std::fmt::Formatter;
use pest_derive::Parser;
use super::*;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct CurryParser;


pub trait PairsHelper<'a> : Iterator + Sized {
    fn unique_pair(&mut self) -> Result<Pair<'a, Rule>, IllegalSourceState>;
    fn expect_unique_pair(&mut self) -> Pair<'a, Rule>;
}

impl<'a> PairsHelper<'a> for Pairs<'a,Rule> {
    fn unique_pair(&mut self) -> Result<Pair<'a, Rule>, IllegalSourceState> {
        match self.next() {
            Some(first) => {
                match self.next() {
                    Some(_second) => {
                        Err(IllegalSourceState::UniqueConstraintViolation)
                    }
                    None => {
                        Ok(first)
                    }
                }
            }
            None => Err(IllegalSourceState::UniqueConstraintViolation),
        }
    }
    fn expect_unique_pair(&mut self) -> Pair<'a, Rule> {
        self.unique_pair().expect("no unique pair contained")
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_function_call() {
        CurryParser::parse(
            Rule::function_call,
            r#"printf("Hello World!")"#
        ).unwrap()
            .unique_pair().unwrap();
    }

    #[test]
    fn it_parses_assignment() {
        CurryParser::parse(
            Rule::assignment,
            r#"name = "chuck norris""#
        ).unwrap()
            .unique_pair().unwrap();
    }
}