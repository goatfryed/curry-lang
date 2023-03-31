use std::convert::{TryFrom, TryInto};
use anyhow::{Context, Error};
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};
use pest::iterators::Pair;
use crate::ll_code_gen::function_generator::FunctionGenerator;
use crate::parser::curry_pest::{Rule};
use crate::parser::{InvalidParserState};
use crate::parser::ast::PairHelper;

pub struct Assignment<'i> {
    pub symbol_ref: SymbolRef,
    pub expression: Pair<'i,Rule>,
}

pub struct SymbolRef{
    pub name: String
}
pub struct StringValue {
    pub value: String,
}

impl<'i> Assignment<'i> {
    pub fn process<'gen>(self, scope: &FunctionGenerator<'gen,'_,'_>) -> anyhow::Result<BasicValueEnum<'gen>> {
        let name = self.symbol_ref.name;
        if scope.symbols.borrow().contains_key(name.as_str()) {
            return Err(Error::msg(format!("variable '{}' is already defined", name)))
        }

        let pointer = match self.expression.as_rule() {
            Rule::value => {
                let value_pair = self.expression.unique_inner().unwrap();
                match value_pair.as_rule() {
                    Rule::string_literal => {
                        let string_value = StringValue::try_from_value(value_pair).unwrap();
                        generate_string_literal_expression(name.as_str(), string_value, scope)
                    },
                    it => unreachable!("unexpected assignment->expression {}", it),
                }
            },
            Rule::function_call => {
                let value = scope.process_function_call(self.expression).context("function call in assignment requires return value")?;
                let pointer = scope.builder.build_alloca(value.get_type(), name.as_str());
                
                match value {
                    BasicValueEnum::PointerValue(value) => { scope.builder.build_store(pointer, value) },
                    BasicValueEnum::IntValue(value) => { scope.builder.build_store(pointer, value) },
                    BasicValueEnum::ArrayValue(value) => { scope.builder.build_store(pointer, value) },
                    BasicValueEnum::FloatValue(value) => { scope.builder.build_store(pointer, value) },
                    BasicValueEnum::StructValue(value) => { scope.builder.build_store(pointer, value) },
                    BasicValueEnum::VectorValue(value) => { scope.builder.build_store(pointer, value) },
                };

                pointer
            },
            it => unreachable!("unexpected assignment {}", it)
        };

        let basic_value = pointer.as_basic_value_enum();
        let mut symbols = scope.symbols.borrow_mut();
        symbols.insert(name, pointer);
        
        Ok(basic_value)
    }
}

fn generate_string_literal_expression<'ctx>(name: &str, value: StringValue, scope: &FunctionGenerator<'ctx,'_,'_>) -> PointerValue<'ctx> {
    let array_value = scope.parent.parent.context.const_string(value.value.as_bytes(), true);
    let pointer = scope.builder.build_alloca(array_value.get_type(), name);
    scope.builder.build_store(pointer, array_value);

    pointer
}

impl<'i> TryFrom<Pair<'i, Rule>> for Assignment<'i> {
    type Error = Error;

    fn try_from(value: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        match value.as_rule() {
            Rule::assignment => {
                let mut pairs = value.into_inner();
                let symbol_ref : SymbolRef   = pairs.next().expect("assignment->symbol_name missing").try_into()?;
                let expression = pairs.next().expect("assignment->expression missing");
                Ok(Assignment { symbol_ref, expression })
            }
            rule => Err(InvalidParserState::illegal_rule(rule)).context("cst -> ast: assignment")
        }

    }
}

impl <'a>  StringValue {
    fn try_from_value(value: Pair<'a, Rule>) -> Result<Self, Error> {
        value.unique_inner().context("read string value")
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

