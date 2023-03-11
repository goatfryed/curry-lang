use std::fs::read_to_string;
use std::iter::empty;
use std::path::Path;
use failure::Error;
use inkwell::{AddressSpace, OptimizationLevel};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::{BasicValue, BasicValueEnum};
use pest::iterators::{Pair, Pairs};
use crate::parser;
use pest::Parser;
use single::Single;

type MainFn = unsafe extern "C" fn();

#[derive(Debug)]
pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub entry: Module<'ctx>,
    builder: Builder<'ctx>,
    engine: ExecutionEngine<'ctx>
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &Context) -> CodeGen {
        let system = context.create_module("system");
        let builder = context.create_builder();
        let engine =
            system.create_jit_execution_engine(OptimizationLevel::None)
            .expect("failed to create execution engine");
        return  CodeGen { context, entry: system, builder, engine };
    }

    pub fn compile_source<P: AsRef<Path>>(&self, path: P) -> Result<(),Error> {
        let raw_source = read_to_string(path)?;

        let root: Pair<parser::Rule> = parser::CurryParser::parse(
            parser::Rule::statement,
            raw_source.as_str()
        )?.single()?;

        self.add_builtins();
        self.begin_main();

        match root.as_rule() {
            parser::Rule::assignment => println!("got an assignment"),
            parser::Rule::function_call => {
                let mut inner = root.into_inner();
                let symbol_ref = inner.next().expect("function call requires symbol ref").as_str();
                let args = inner.next().map(|args| self.build_fn_args(args.into_inner())).unwrap_or(Vec::new());
                self.builder.build_call(
                    self.entry.get_function(symbol_ref).expect(&*format!("{} not defined", symbol_ref)),
                    args.as_ref(),
                    symbol_ref
                );
            },
            _ => println!("{:?}", root)
        }

        return Ok(());
    }

    fn build_fn_args(&self, pairs: Pairs<parser::Rule>) -> Vec<BasicValueEnum<'ctx>> {
        return pairs.into_iter()
            .map(|arg| self.build_fn_arg(arg))
            .collect::<Vec<BasicValueEnum>>();
    }

    fn build_fn_arg(&self, pair: Pair<parser::Rule>) -> BasicValueEnum {
        match pair.as_rule() {
            parser::Rule::expression => {},
            _ => panic!("unsupported pair {:?}", pair)
        }
        let pair = pair.into_inner().single().expect("expression without a single token below");
        return match pair.as_rule() {
            parser::Rule::value => self.builder
                .build_global_string_ptr(pair.as_str(), "arg")
                .as_basic_value_enum()
            ,
            parser::Rule::function_call => todo!("function call as function argument"),
            _ => panic!("unsupported pair {:?}", pair)
        }
    }

    pub fn compile_printer(&self) -> JitFunction<'ctx, MainFn> {

        self.add_builtins();
        self.begin_main();

        let format = self.builder.build_global_string_ptr("Hello World! Greetings from %s!\n", "format");
        let value = self.builder.build_global_string_ptr("curry-lang", "value");

        self.builder.build_call(
            self.entry.get_function("printf").expect("printf not defined"),
            &[format.as_basic_value_enum(), value.as_basic_value_enum()],
            "printf"
        );
        self.builder.build_return(None);

        let main = unsafe { self.engine.get_function("main").expect("unable to compile").to_owned() };
        return main;
    }

    fn begin_main(&self) {
        let type_main = self.context.void_type().fn_type(&[], false);
        let main = self.entry.add_function("main", type_main, None);
        let entry = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(entry);
    }

    fn add_builtins(&self) {
        let char_array = self.context.i8_type().ptr_type(AddressSpace::Generic);
        let param_types = &[char_array.into()];
        let void = self.context.void_type();
        let type_printf = void.fn_type(param_types, true);
        self.entry.add_function("printf", type_printf, None);
    }
}