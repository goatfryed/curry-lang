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
use crate::ll_code_gen::function_generator::libc::declare_libc_builtin;
use crate::parser::ast::*;

mod function_generator;

const ENTRY_BLOCK_NAME: &str = "entry";
const MAIN_FN_NAME: &str = "main";

#[derive(Debug)]
pub struct LLIRCodeGenerator<'a: 'b, 'b> {
    pub context: &'a Context,
    pub modules: HashMap<String, Rc<Module<'b>>>
}

#[derive(Debug)]
pub struct ModuleGenerator<'a: 'b, 'b> {
    pub context: &'a Context,
    pub module: &'b Module<'a>,
    pub builder: &'b Builder<'a>,
}

impl <'a, 'b> LLIRCodeGenerator<'a, 'b> {
    pub fn new(context: &'a Context) -> LLIRCodeGenerator<'a, 'b> {
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
        self.create_module(MAIN_FN_NAME, |module_gen: &ModuleGenerator| {
            declare_libc_builtin(module_gen);
            module_gen.build_fn(MAIN_FN_NAME, statements);
        })
    }

    pub fn create_module<F>
    (
        &mut self,
        name: &str,
        build_module: F
    ) -> anyhow::Result<()>
        where F: FnOnce(&ModuleGenerator)
    {
        let module = Rc::new(self.context.create_module(name));
        let builder = self.context.create_builder();
        let module_gen = ModuleGenerator {module: &module, context: self.context, builder: &builder};
        build_module(&module_gen);
        self.modules.insert(name.to_string(), module);

        Ok(())
    }
}

impl <'a: 'b, 'b> ModuleGenerator<'b, '_> {
    fn build_fn(&self, name: &str, statements: Vec<Statement>) {
        function_generator::generate(self, name, statements);
    }
}
