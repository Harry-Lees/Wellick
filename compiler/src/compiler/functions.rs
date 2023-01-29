use crate::parser::ast;

use cranelift::{
    codegen,
    prelude::{AbiParam, InstBuilder},
};
use cranelift_codegen::ir::types;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_native::builder as host_isa_builder;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs::File;
use std::io::prelude::Write;

pub struct Compiler {
    builder_context: FunctionBuilderContext,
    codegen_context: codegen::Context,
    data_context: DataContext,
    module: cranelift_object::ObjectModule,
}

impl Compiler {
    pub fn new() -> Self {
        let flag_builder = codegen::settings::builder();
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

    // Compile a parsed AST
    pub fn compile(mut self, code: Vec<ast::Item>) -> Result<(), String> {
        self.translate(code);

        // Finish
        match self.finish() {
            Ok(_) => {}
            Err(_) => return Err("Failed to emit bytecode".to_string()),
        }

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

    fn translate(&mut self, code: Vec<ast::Item>) {
        for item in code {
            match item {
                ast::Item::FnDecl(node) => self.translate_decl(node),
            }
        }
    }

    fn translate_decl(&mut self, node: ast::FnDecl) {
        // Define an integer type for use in the program.
        let type_isize = self.module.target_config().pointer_type();
        let type_f32 = types::F32;
        let type_f64 = types::F64;

        for arg in node.args {
            let arg_type = match arg.t {
                ast::Type::isize => type_isize,
                ast::Type::f32 => type_f32,
                ast::Type::f64 => type_f64,
            };
            self.codegen_context
                .func
                .signature
                .params
                .push(AbiParam::new(arg_type))
        }

        self.codegen_context
            .func
            .signature
            .returns
            .push(AbiParam::new(type_isize));

        let mut function_builder =
            FunctionBuilder::new(&mut self.codegen_context.func, &mut self.builder_context);
        let entry_block = function_builder.create_block();
        function_builder.switch_to_block(entry_block);
        function_builder.append_block_params_for_function_params(entry_block);

        let value = function_builder.ins().iconst(type_isize, 0);
        function_builder.ins().return_(&[value]);

        function_builder.seal_block(entry_block);
        function_builder.finalize();
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

        println!("{}", self.codegen_context.func.display());
        // Clear the function context ready for the next function
        self.module.clear_context(&mut self.codegen_context);
    }
}
