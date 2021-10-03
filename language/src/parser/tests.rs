use super::*;
use single::Single;

#[test]
fn it_parses_assignment_statement() {
    let parsed: Vec<_> = CurryParser::parse(
        Rule::statement,
        r#"menu12a = "spicy""#
    ).unwrap().next().unwrap()
        .into_inner().into_iter().collect();
    assert_eq!(1, parsed.len());
    let statement = &parsed[0];
    assert_eq!(Rule::assignment, statement.as_rule());
}

mod function {
    use super::*;

    #[test]
    fn it_parses() {
        let function_call = CurryParser::parse(
            Rule::function_call,
            r#"println("Hello World!")"#
        ).unwrap().single().unwrap();

        let mut tokens:Vec<_> = function_call.into_inner().collect();
        assert_eq!(2, tokens.len());
        assert_eq!(Rule::symbol_ref, tokens.remove(0).as_rule());

        let args = tokens.remove(0);
        assert_eq!(Rule::fn_args, args.as_rule());

        let arg = args.into_inner().single().unwrap();
        assert_eq!(Rule::expression, arg.as_rule());
    }

    #[test]
    fn its_a_statement() {
        let expression = CurryParser::parse(
            Rule::statement,
            r#"println("Hello World!")"#
        ).unwrap().single().unwrap()
            .into_inner()
            .single().unwrap();
        assert_eq!(Rule::expression, expression.as_rule());
        let function_call = expression.into_inner().single().unwrap();
        assert_eq!(Rule::function_call, function_call.as_rule());
    }
}