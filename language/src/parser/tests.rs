use super::*;

#[test]
fn it_parses_successful() {
    CurryParser::parse(Rule::statement, "Mushroom12 = \"ThisAndThat\"").unwrap();
}

#[test]
#[should_panic]
fn it_yields_error() {
    CurryParser::parse(Rule::statement, "Mushroom12 = \"ThisAndThat").unwrap();
}