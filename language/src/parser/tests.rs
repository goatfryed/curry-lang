use super::*;

#[test]
fn it_parses_assignment_statement() {
    CurryParser::parse(
        Rule::statement, r##"
        menu12a = "spicy"
    "##).unwrap();
}