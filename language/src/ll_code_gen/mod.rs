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

        self.compile_source(input)
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
                    .context("create main module")?
            },
            Rule::program => {},
            it => unreachable!("unexpected rule {}", it)
        };

        Ok(())
    }

    fn create_script(&mut self, statements: Vec<Statement>) -> Result<()> {
        let name = MAIN_FN_NAME;
        let module_gen = self.create_module_generator(name);
        module_gen.declare_libc_builtin();
        module_gen.generate_function(MAIN_FN_NAME, statements);

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

    pub fn generate_function(&self, name: &str, statements: Vec<Statement>) {
        FunctionGenerator::generate(self, name, statements);
    }

}
