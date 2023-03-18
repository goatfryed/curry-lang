use std::convert::TryFrom;
use std::fmt;
use super::*;
use pest::iterators::Pair;
use pest::Span;
use crate::parser::pest_utils::{InvalidParserState};

#[derive(Debug)]
pub struct Statement<'a> {
    pub span: Span<'a>,
    pub kind: StatementKind<'a>,
}

#[derive(Debug)]
pub enum StatementKind<'a> {
    Assignment(Pair<'a,Rule>),
    FunctionCall(Pair<'a,Rule>),
}


impl <'a> TryFrom<Pair<'a,Rule>> for Statement<'a> {
    type Error = InvalidParserState;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let span = pair.as_span();
        let kind = match pair.as_rule() {
            Rule::assignment => Ok(StatementKind::Assignment(pair)),
            Rule::function_call => Ok(StatementKind::FunctionCall(pair)),
            rule => Err(
                InvalidParserState::illegal_rule(rule, "Tried to create statement".to_string())
                    .into()
            ),
        };
        kind.map(|kind| Statement {kind, span})
    }
}

impl <'a> fmt::Display for StatementKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}