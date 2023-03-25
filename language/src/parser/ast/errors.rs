use thiserror::Error;
use crate::parser::curry_pest::Rule;

#[derive(Error,Debug)]
pub enum IllegalSourceState {
    #[error("Expected exactly one inner pair")]
    UniqueConstraintViolation,
    #[error("{rule} is not allowed")]
    IllegalRule {
        rule: Rule,
    },
}