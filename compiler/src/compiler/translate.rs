use super::ast;
use super::variables;

use cranelift::prelude::AbiParam;
use cranelift::prelude::InstBuilder;
use cranelift_codegen::ir::{entities::Value, types, InstBuilderBase};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;

/// Module to translate AST into Cranelift IR constructs.
pub struct FunctionTranslator<'a, 'b: 'a> {
    pub(crate) builder: FunctionBuilder<'b>,
    pub(crate) variables: HashMap<String, variables::Variable>,
    pub(crate) module: &'a mut ObjectModule,
}

impl<'a, 'b> FunctionTranslator<'a, 'b> {
    pub fn new(
        builder: FunctionBuilder<'b>,
        variables: HashMap<String, variables::Variable>,
        module: &'b mut ObjectModule,
    ) -> Self {
        Self {
            builder,
            variables,
            module,
        }
    }

    pub fn translate_stmt(&mut self, stmt: &ast::Stmt) -> Value {
        match stmt {
            ast::Stmt::Assign(expr) => self.translate_assign(expr),
            ast::Stmt::If(condition, body, else_body) => {
                todo!("If stmt not implemented");
            }
            ast::Stmt::Return(expr) => self.translate_return(expr),
        }
    }

    pub fn translate_expr(&mut self, expr: &ast::Expression) -> Value {
        match expr {
            ast::Expression::Call(val) => self.translate_call(val),
            ast::Expression::Literal(val) => match val {
                ast::Value::Float(value) => self.builder.ins().f32const(*value),
                ast::Value::Integer(value) => self.builder.ins().iconst(types::I32, *value as i64),
            },
            ast::Expression::Identifier(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");
                self.builder.use_var(var.reference)
            }
        }
    }

    fn translate_call(&mut self, expr: &ast::Call) -> Value {
        println!("{:?}", self.module.get_name(expr.func.as_str()));
        let mut sig = self.module.make_signature();
        let mut arg_values = Vec::new();
        for arg in &expr.args {
            let func_arg = self.translate_expr(arg);
            arg_values.push(func_arg);
            // let arg_type = self.builder.ins().data_flow_graph().value_type(func_arg);
            let arg_type = self.builder.func.dfg.value_type(func_arg);
            sig.params.push(AbiParam::new(arg_type));
        }
        sig.returns.push(AbiParam::new(types::I32));
        let callee = self
            .module
            .declare_function(&expr.func, Linkage::Import, &sig)
            .expect("Unable to declare function");
        let local_callee = self.module.declare_func_in_func(callee, self.builder.func);
        let call = self.builder.ins().call(local_callee, &arg_values);
        self.builder.inst_results(call)[0]
    }

    fn translate_return(&mut self, expr: &ast::Expression) -> Value {
        let value = self.translate_expr(expr);
        self.builder.ins().return_(&[value]);
        value
    }

    fn translate_assign(&mut self, expr: &ast::Assignment) -> Value {
        let name = &expr.target.ident;
        let value = self.translate_expr(&expr.value);
        let var = self
            .variables
            .get(name)
            .expect(format!("No variable named {}", name).as_str());
        self.builder.def_var(var.reference, value);
        value
    }
}
