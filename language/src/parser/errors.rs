pub use crate::parser::ast::errors::IllegalSourceState;

use thiserror::Error;
use crate::parser::curry_pest::Rule;


#[derive(Error,Debug)]
pub enum InvalidParserState {
    #[error("{0}")]
    IllegalSourceState(IllegalSourceState),
}

impl InvalidParserState {
    pub fn illegal_rule_with_context(rule: Rule, _context: String) -> IllegalSourceState {
        IllegalSourceState::IllegalRule { rule }
    }
    pub fn illegal_rule(rule: Rule) -> IllegalSourceState {
        IllegalSourceState::IllegalRule { rule }
    }
}

impl From<IllegalSourceState> for InvalidParserState {
    fn from(value: IllegalSourceState) -> Self {
        InvalidParserState::IllegalSourceState(value)
    }
}