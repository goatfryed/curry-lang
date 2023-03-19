use std::convert::TryFrom;
use std::fmt;
use pest::iterators::Pair;
use pest::Span;
use crate::parser::{Rule,InvalidParserState};

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

#[cfg(test)]
mod test {
    use super::*;
    use pest::Parser;
    use std::convert::TryInto;
    use crate::parser::{CurryParser,PairsHelper};

    #[test]
    fn assignment_is_statement() {
        let statement: Statement = CurryParser::parse(
            Rule::statement,
            r#"menu12a = "spicy""#
        ).unwrap()
            .unique_pair().unwrap()
            .try_into().unwrap();

        match statement.kind {
            StatementKind::Assignment(_pair) => {},
            _ => panic!("unexpected stmt type {}", statement.kind)
        }
    }

    #[test]
    fn function_call_is_statement() {
        let statement: Statement = CurryParser::parse(
            Rule::statement,
            r#"printf("Hello World!")"#
        ).unwrap()
            .unique_pair().unwrap()
            .try_into().unwrap();

        match statement.kind {
            StatementKind::FunctionCall(_pair) => {},
            _ => panic!("unexpected stmt type {}", statement.kind)
        }
    }

}