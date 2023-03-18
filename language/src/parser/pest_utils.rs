use std::fmt;
use std::fmt::{Formatter};
use thiserror::Error;
use pest::iterators::{Pair, Pairs};
use super::*;

#[derive(Error,Debug)]
pub enum InvalidParserState {
    #[error("{0}")]
    IllegalSourceState(IllegalSourceState),
}

#[derive(Error,Debug)]
pub enum IllegalSourceState {
    #[error("Expected exactly one inner pair")]
    UniqueConstraintViolation,
    #[error("{rule} is not allowed here: {context}")]
    IllegalRule {
        rule: Rule,
        context: String,
    },

}

impl InvalidParserState {
    pub fn illegal_rule(rule: Rule, context: String) -> IllegalSourceState {
        IllegalSourceState::IllegalRule { rule, context }
    }
}

impl From<IllegalSourceState> for InvalidParserState {
    fn from(value: IllegalSourceState) -> Self {
        InvalidParserState::IllegalSourceState(value)
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

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}