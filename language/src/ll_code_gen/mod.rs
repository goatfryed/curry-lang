use std::collections::HashMap;
use anyhow::*;
use anyhow::Context as AnyhowContext;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicMetadataValueEnum};
use pest::iterators::{Pairs};
use std::convert::TryInto;
use std::fs::read_to_string;
use std::path::Path;

use std::rc::Rc;
use crate::ll_code_gen::function_generator::FunctionGenerator;
use crate::ll_code_gen::function_generator::libc::declare_libc_builtin;
use crate::parser::ast::*;

mod function_generator;
mod assignment;
mod expression;

const ENTRY_BLOCK_NAME: &str = "entry";
const MAIN_FN_NAME: &str = "main";

#[derive(Debug)]
pub struct LLIRCodeGenerator<'gen> {
    pub context: &'gen Context,
    pub modules: HashMap<String, Rc<Module<'gen>>>
}

impl <'gen> LLIRCodeGenerator<'gen> {
    pub fn new(context: &'gen Context) -> LLIRCodeGenerator<'gen> {
        LLIRCodeGenerator {
            context,
            modules: HashMap::new(),
        }
    }

    pub fn  compile_source_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let input = read_to_string(&path)
            .with_context(|| format!("reading {}", path.as_ref().display()))?;

        self.compile_source(input).context("compile source from file")
    }

    pub fn compile_source(&mut self, input: String) -> Result<()> {

        let source = parse_to_ast(input.as_ref()).context("compile source")?;

        match source.as_rule() {
            Rule::script => {
                let statements: Result<Vec<Statement>> = source.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::EOI)
                    .map(|pair| -> Result<Statement,Error> {
                        pair.try_into().context("couldn't parse stmt")
                    })
                    .collect();

                self.create_script(statements?)
                    .context("create script")?
            },
            Rule::program => self.create_program(source).context("create program")?,
            it => unreachable!("unexpected rule {}", it)
        };

        Ok(())
    }

    fn create_program(&mut self, program: Pair<Rule>) -> Result<()> {
        let mut entry: Option<&str> = None;
        let module_gen = self.create_module_generator(MAIN_FN_NAME);

        module_gen.declare_libc_builtin();

        let pairs = program.into_inner();
        for pair in pairs {
            match pair.as_rule() {
                Rule::entry_definition => {
                    match entry {
                        Some(_) => Err(Error::msg("encountered multiple entry definitions"))?,
                        None => entry = Some(pair.unique_inner()?.as_str()),
                    }
                },
                Rule::function_declaration => {
                    module_gen.process_function_declaration(pair)?
                },
                Rule::EOI => {},
                it => unreachable!("unexpected rule {}", it),
            }
        }

        match entry {
            Some(entry_fn_name) => module_gen.create_program_main(entry_fn_name),
            None => unreachable!("entry definition missing"),
        }

        Ok(())
    }

    fn create_script(&mut self, statements: Vec<Statement>) -> Result<()> {
        let name = MAIN_FN_NAME;
        let module_gen = self.create_module_generator(name);
        module_gen.declare_libc_builtin();
        module_gen.generate_function_from_statements(MAIN_FN_NAME, statements);

        Ok(())
    }

    fn create_module_generator<'module>(&'module mut self, name: &str) -> ModuleGenerator<'gen,'module>
        where 'gen: 'module
    {
        let module = Rc::new(self.context.create_module(name));
        self.modules.insert(name.to_string(), module.clone());
        let module_gen = ModuleGenerator::create(self, module);
        module_gen
    }
}


#[derive(Debug)]
pub struct ModuleGenerator<'gen: 'module, 'module> {
    pub parent: &'module LLIRCodeGenerator<'gen>,
    pub module: Rc<Module<'gen>>,
    pub builder: Builder<'gen>,
}

impl <'gen: 'module, 'module> ModuleGenerator<'gen, 'module> {

    pub fn create(parent: &'module LLIRCodeGenerator<'gen>, module: Rc<Module<'gen>>) -> Self {
        let builder = parent.context.create_builder();
        ModuleGenerator {module, parent, builder}
    }

    pub fn declare_libc_builtin(&self) {
        declare_libc_builtin(self);
    }

    pub fn create_program_main(&self, entry_fn_name: &str) {
        let function = FunctionGenerator::declare_void_fn(self, MAIN_FN_NAME);
        let fn_gen = FunctionGenerator::create_generator(self, function);
        fn_gen.create_function_call(entry_fn_name, Vec::new());
        fn_gen.complete();
    }

    pub fn generate_function_from_statements(&self, name: &str, statements: Vec<Statement>) {
        FunctionGenerator::generate(self, name, statements);
    }
    pub fn process_function_declaration(&self, pair: Pair<Rule>) -> Result<()> {
        let _line_col = pair.line_col();
        let mut pairs = pair.into_inner();
        let name = pairs.next().context("function name")?.as_str();
        let _args = pairs.next().context("function args")?.into_inner();
        let body = pairs.next()
            .context("function body")?
            .into_inner();

        let function = FunctionGenerator::declare_void_fn(self, name);
        let fn_gen = FunctionGenerator::create_generator(self, function);

        body.map( |it| it.try_into().context("function body"))
            .try_for_each(|it|
                it.map(|pair|
                    fn_gen.add_statement(pair)
                )
            )?;

        fn_gen.complete();

        Ok(())
    }

}
