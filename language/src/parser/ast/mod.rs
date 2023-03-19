use thiserror::Error;
use crate::parser::*;

pub mod statement;


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

impl From<IllegalSourceState> for InvalidParserState {
    fn from(value: IllegalSourceState) -> Self {
        InvalidParserState::IllegalSourceState(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::PairsHelper;

    #[test]
    fn it_parses_function_call() {
        CurryParser::parse(
            Rule::function_call,
            r#"printf("Hello World!")"#
        ).unwrap()
            .unique_pair().unwrap();
    }
}
