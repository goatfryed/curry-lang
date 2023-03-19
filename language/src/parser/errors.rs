pub use crate::parser::ast::errors::IllegalSourceState;

use thiserror::Error;
use crate::parser::curry_pest::Rule;


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

impl From<IllegalSourceState> for InvalidParserState {
    fn from(value: IllegalSourceState) -> Self {
        InvalidParserState::IllegalSourceState(value)
    }
}