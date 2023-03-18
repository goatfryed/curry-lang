use super::*;
use itertools::{Itertools};
use std::error::Error;
use std::convert::TryInto;
use crate::parser::ast::{Statement, StatementKind};
use crate::parser::pest_utils::PairsHelper;


mod basic {
    use super::*;

    #[test]
    fn it_parses_function_call() -> Result<(), Box<dyn Error>> {
        CurryParser::parse(
            Rule::function_call,
            r#"printf("Hello World!")"#
        )?.exactly_one()?;
        Ok(())
    }
}

mod statements {
    use super::*;

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
