use std::collections::HashMap;
use anyhow::*;
use anyhow::Context as AnyhowContext;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicMetadataValueEnum, BasicValue};
use pest::iterators::{Pair, Pairs};
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

#[derive(Debug)]
pub struct ModuleGenerator<'gen: 'module, 'module> {
    pub parent: &'module LLIRCodeGenerator<'gen>,
    pub module: Rc<Module<'gen>>,
    pub builder: Builder<'gen>,
}

impl <'gen: 'module, 'module> ModuleGenerator<'gen, 'module> {

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

        self.compile_source(input)
    }

    pub fn compile_source(&mut self, input: String) -> Result<()> {

        let source = parse_to_ast(input.as_ref())?;
        self.create_main_module(source)
            .context("create main module")?;

        Ok(())
    }

    fn create_main_module(&mut self, statements: Vec<Statement>) -> anyhow::Result<()> {
        let name = MAIN_FN_NAME;
        let module = Rc::new(self.context.create_module(name));
        self.modules.insert(name.to_string(), module.clone());
        let builder = self.context.create_builder();
        let module_gen = Box::new(ModuleGenerator {module, parent: self, builder});
        declare_libc_builtin(module_gen.as_ref());
        FunctionGenerator::generate(&module_gen, MAIN_FN_NAME, statements);

        Ok(())
    }
}
