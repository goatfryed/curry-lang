use std::fs::read_to_string;
use std::path::Path;
use failure::Error;
use inkwell::{AddressSpace};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValue, BasicMetadataValueEnum};
use pest::iterators::{Pair, Pairs};
use crate::parser::curry_pest;
use pest::Parser;
use itertools::Itertools;
use std::convert::TryInto;

#[derive(Debug)]
pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub entry: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &Context) -> CodeGen {
        let system = context.create_module("system");
        let builder = context.create_builder();

        CodeGen { context, entry: system, builder }
    }

    pub fn compile_source<P: AsRef<Path>>(&self, path: P) -> Result<(),Error> {
        let raw_source = read_to_string(path).unwrap();

        let root = curry_pest::CurryParser::parse(
            curry_pest::Rule::statement,
            raw_source.as_str()
        ).unwrap().exactly_one().unwrap();

        self.add_builtins();
        self.begin_main();

        match root.as_rule() {
            curry_pest::Rule::assignment => println!("got an assignment"),
            curry_pest::Rule::function_call => {
                let mut inner = root.into_inner();
                let symbol_ref = inner.next().expect("function call requires symbol ref").as_str();
                let args = inner.next().map(|args| self.build_fn_args(args.into_inner())).unwrap_or(Vec::new());
                self.builder.build_call(
                    self.entry.get_function(symbol_ref).unwrap_or_else(|| panic!("{} not defined", symbol_ref)),
                    args.as_ref(),
                    symbol_ref
                );
                self.builder.build_return(None);
            },
            _ => println!("{:?}", root)
        }

        Ok(())
    }

    fn build_fn_args(&self, pairs: Pairs<curry_pest::Rule>) -> Vec<BasicMetadataValueEnum<'ctx>> {
        pairs.into_iter()
            .map(|arg| self.build_fn_arg(arg))
            .collect::<Vec<BasicMetadataValueEnum<'ctx>>>()
    }

    fn build_fn_arg(&self, pair: Pair<curry_pest::Rule>) -> BasicMetadataValueEnum<'ctx> {
        match pair.as_rule() {
            curry_pest::Rule::expression => {},
            _ => panic!("unsupported pair {:?}", pair)
        }
        let pair = pair.into_inner().exactly_one().expect("expression without a single token below");
        return match pair.as_rule() {
            curry_pest::Rule::value => self.builder
                .build_global_string_ptr(pair.as_str(), "arg")
                .as_basic_value_enum().try_into().unwrap()
            ,
            curry_pest::Rule::function_call => todo!("function call as function argument"),
            _ => panic!("unsupported pair {:?}", pair)
        }
    }

    fn begin_main(&self) {
        let type_main = self.context.void_type().fn_type(&[], false);
        let main = self.entry.add_function("main", type_main, None);
        let entry = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(entry);
    }

    fn add_builtins(&self) {
        let char_array = self.context.i8_type().ptr_type(AddressSpace::default());
        let param_types = &[char_array.into()];
        let void = self.context.void_type();
        let type_printf = void.fn_type(param_types, true);
        self.entry.add_function("printf", type_printf, None);
    }
}