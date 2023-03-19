pub mod ast;

pub use pest::Parser;
pub use ast::IllegalSourceState;

use pest_derive::Parser;
use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;
use pest::iterators::*;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct CurryParser;

#[derive(Error,Debug)]
pub enum InvalidParserState {
    #[error("{0}")]
    IllegalSourceState(IllegalSourceState),
}

impl InvalidParserState {
    pub fn illegal_rule(rule: Rule, context: String) -> IllegalSourceState {
        IllegalSourceState::IllegalRule { rule, context }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub trait PairsHelper<'a> : Iterator + Sized {
    fn unique_pair(self) -> Result<Pair<'a, Rule>, IllegalSourceState>;
}

impl<'a> PairsHelper<'a> for Pairs<'a,Rule> {
    fn unique_pair(mut self) -> Result<Pair<'a, Rule>, IllegalSourceState> {
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
}