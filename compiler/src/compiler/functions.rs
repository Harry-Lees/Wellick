use crate::parser::ast;

use cranelift::{
    codegen,
    prelude::{AbiParam, InstBuilder},
};
use cranelift_codegen::ir::entities::Value;
use cranelift_codegen::ir::{types, Block};
use cranelift_frontend::Variable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_native::builder as host_isa_builder;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Write;

fn to_cranelift_type(t: &ast::EmptyType) -> types::Type {
    match t {
        ast::EmptyType::Float => types::F64,
        ast::EmptyType::Integer => types::I64,
    }
}

fn value_type(t: &ast::Value) -> types::Type {
    match t {
        ast::Value::Float(_) => types::F64,
        ast::Value::Integer(_) => types::I64,
    }
}

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

    /// Compile a parsed AST
    pub fn compile(mut self, code: Vec<ast::Item>) -> Result<(), String> {
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

    fn translate(&mut self, code: Vec<ast::Item>) {
        for item in code {
            match item {
                ast::Item::FnDecl(node) => self.translate_decl(node),
            }
        }
    }

    fn translate_decl(&mut self, node: ast::FnDecl) {
        // Define function arguments
        for arg in &node.args {
            let t = to_cranelift_type(&arg.t);
            self.codegen_context
                .func
                .signature
                .params
                .push(AbiParam::new(t))
        }

        if let Some(ret_type) = &node.ret_type {
            let t = to_cranelift_type(&ret_type);
            self.codegen_context
                .func
                .signature
                .returns
                .push(AbiParam::new(t));
        }

        let mut function_builder =
            FunctionBuilder::new(&mut self.codegen_context.func, &mut self.builder_context);

        let entry_block = function_builder.create_block();
        function_builder.switch_to_block(entry_block);
        function_builder.append_block_params_for_function_params(entry_block);

        let int = self.module.target_config().pointer_type();
        let mut vars = declare_variables(
            int,
            &mut function_builder,
            &node.args,
            &node.body,
            entry_block,
        );

        for expr in &node.body {
            if let ast::Expression::Assign(assign) = expr {
                translate_assign(
                    assign.target.ident.clone(),
                    &assign,
                    &mut function_builder,
                    &mut vars,
                );
            }
        }

        if let Some(ret_type) = &node.ret_type {
            let t = to_cranelift_type(&ret_type);
            let value = function_builder.ins().iconst(t, 0);
            function_builder.ins().return_(&[value]);
        }

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

fn translate_assign(
    name: String,
    expr: &ast::Assignment,
    function_builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
) -> Value {
    let var = variables
        .get(&name)
        .expect(format!("No variable named {}", name).as_str());
    println!("var: {name}, {var}");
    // let value = match expr.value {
    //     ast::Atom::Name(_) => {
    //         todo!();
    //     }
    //     ast::Atom::Constant(value) => value,
    // };
    let value = function_builder.ins().iconst(types::I64, 0);
    function_builder.def_var(*var, value);
    value
}

fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder,
    args: &Vec<ast::FnArg>,
    stmts: &Vec<ast::Expression>,
    entry_block: Block,
) -> HashMap<String, Variable> {
    let mut variables = HashMap::new();
    let mut index = 0;

    for (i, arg) in args.iter().enumerate() {
        let val = builder.block_params(entry_block)[i];
        let var = declare_variable(int, builder, &mut variables, &mut index, &arg.name);
        builder.def_var(var, val);
    }

    for expr in stmts {
        declare_variables_in_stmt(builder, &mut variables, &mut index, expr);
    }

    variables
}

/// Declare a single variable declaration.
fn declare_variable(
    var_type: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    name: &str,
) -> Variable {
    let var = Variable::from_u32(*index as u32);
    if !variables.contains_key(name) {
        variables.insert(name.into(), var);
        builder.declare_var(var, var_type);
        *index += 1;
    }
    var
}

/// Recursively descend through the AST, translating all implicit
/// variable declarations.
fn declare_variables_in_stmt(
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    expr: &ast::Expression,
) -> Variable {
    match expr {
        ast::Expression::Assign(ref assignment) => declare_variable(
            value_type(&assignment.value),
            builder,
            variables,
            index,
            assignment.target.ident.as_ref(),
        ),
        ast::Expression::Call(_) => {
            todo!();
        }
        ast::Expression::If(_, _, _) => {
            todo!();
        }
    }
}
