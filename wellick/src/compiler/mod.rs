mod translate;
mod variables;

use crate::parser::ast;

use cranelift::codegen;
use cranelift::prelude::AbiParam;
use cranelift::prelude::Configurable;
use cranelift_codegen::ir::types;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_native::builder as host_isa_builder;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs::File;
use std::io::prelude::Write;
use std::process;

pub struct Compiler {
    builder_context: FunctionBuilderContext,
    codegen_context: codegen::Context,
    data_context: DataContext,
    module: cranelift_object::ObjectModule,
}

impl Default for Compiler {
    /// default constructor for Compiler. Will construct a compiler for the current
    /// machine with default compiler flags.
    fn default() -> Self {
        let mut flag_builder = codegen::settings::builder();
        flag_builder
            .set("opt_level", "speed")
            .expect("Unable to set opt_level");
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = host_isa_builder().expect("host machine is not supported");
        let isa = isa_builder
            .finish(codegen::settings::Flags::new(flag_builder))
            .unwrap();

        let builder = ObjectBuilder::new(
            isa,
            "test".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();

        let module = ObjectModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            codegen_context: module.make_context(),
            data_context: DataContext::new(),
            module,
        }
    }
}

impl Compiler {
    /// Compile a parsed AST
    pub fn compile(mut self, code: Vec<ast::FnDecl>) -> Result<(), String> {
        //self.decl_stdlib();
        self.translate(code);

        // Finish
        match self.finish() {
            Ok(_) => {}
            Err(_) => return Err("Failed to emit bytecode".to_string()),
        }

        Ok(())
    }

    /// Create a zero-initialized data section.
    pub fn create_data(&mut self, name: &str, contents: Vec<u8>) -> Result<(), String> {
        self.data_context.define(contents.into_boxed_slice());
        let id = self
            .module
            .declare_data(name, Linkage::Export, true, false)
            .map_err(|e| e.to_string())?;

        self.module
            .define_data(id, &self.data_context)
            .map_err(|e| e.to_string())?;

        self.data_context.clear();
        Ok(())
    }

    pub fn finish(self) -> Result<(), std::io::Error> {
        let product = self.module.finish();
        let code = product.emit().unwrap();
        println!("writing object file");
        let mut file = File::create("a.out")?;
        file.write_all(&code)?;

        Ok(())
    }

    fn translate(&mut self, code: Vec<ast::FnDecl>) {
        for func in code {
            self.translate_decl(func);
        }
    }

    fn translate_decl(&mut self, node: ast::FnDecl) {
        // Define function arguments
        for arg in &node.args {
            let t = variables::to_cranelift_type(&arg.t);
            self.codegen_context
                .func
                .signature
                .params
                .push(AbiParam::new(t))
        }

        // Define the function return type.
        match &node.ret_type {
            Some(ret_type) => {
                let t = variables::to_cranelift_type(&ret_type);
                self.codegen_context
                    .func
                    .signature
                    .returns
                    .push(AbiParam::new(t));
            }
            None => {
                println!("No return type defined for func {}", &node.name);
                process::exit(1)
            }
        }

        let mut function_builder =
            FunctionBuilder::new(&mut self.codegen_context.func, &mut self.builder_context);

        let entry_block = function_builder.create_block();
        function_builder.switch_to_block(entry_block);
        function_builder.append_block_params_for_function_params(entry_block);

        let vars = variables::declare_variables(&node, &mut function_builder, entry_block);

        let mut translator =
            translate::FunctionTranslator::new(function_builder, vars, &mut self.module);

        for stmt in node.body {
            translator.translate_stmt(&stmt);
        }

        translator.builder.seal_all_blocks();
        translator.builder.finalize();
        println!("{}", self.codegen_context.func.display());
        // Declare a function, has to be done before the function can be
        // Called or defined.
        let function_id = self
            .module
            .declare_function(
                &node.name,
                Linkage::Export,
                &self.codegen_context.func.signature,
            )
            .expect("Failed to create function_id");

        // Define the function on the module
        self.module
            .define_function(function_id, &mut self.codegen_context)
            .expect("Failed to define module function");

        match codegen::verify_function(&self.codegen_context.func, self.module.isa()) {
            Ok(result) => println!("Successfully verified function: {:?}", result),
            Err(result) => println!("An error occurred when verifying function {:?}", result),
        }
        // Clear the function context ready for the next function
        self.module.clear_context(&mut self.codegen_context);
    }
}
