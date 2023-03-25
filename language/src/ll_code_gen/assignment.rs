use std::convert::{TryFrom, TryInto};
use anyhow::{Context, Error};
use pest::iterators::Pair;
use crate::parser::curry_pest::{Rule, PairsHelper};
use crate::parser::{InvalidParserState};

pub struct Assignment {
    pub symbol_ref: SymbolRef,
    pub value: StringValue,
}

pub struct SymbolRef{
    pub name: String
}
pub struct StringValue {
    pub value: String,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Assignment {
    type Error = Error;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match value.as_rule() {
            Rule::assignment => {
                let mut pairs = value.into_inner();
                let symbol_ref : SymbolRef   = pairs.next().context("resolve symbol ref")?.try_into()?;
                let value : StringValue = StringValue::try_from_value(pairs.next().context("resolve value")?)?;
                let it = Assignment { symbol_ref, value };
                Ok(it)
            }
            rule => Err(InvalidParserState::illegal_rule(rule)).context("cst -> ast: assignment")
        }

    }
}

impl <'a>  StringValue {
    fn try_from_value(value: Pair<'a, Rule>) -> Result<Self, Error> {
        value.into_inner().unique_pair().context("get inner string")?
            .into_inner().unique_pair().context("get inner string value")
            .map(|it| { Self { value: it.as_str().to_string() } })
    }
}

impl <'a> TryFrom<Pair<'a,Rule>> for SymbolRef {
    type Error = InvalidParserState;

    fn try_from(value: Pair<'a,Rule>) -> Result<Self, Self::Error> {
        match value.as_rule() {
            Rule::symbol_ref => Ok(Self { name: value.as_str().to_string() }),
            rule => Err(InvalidParserState::illegal_rule(rule).into())
        }
    }
}

