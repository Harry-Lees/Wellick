use super::ast;
use super::variables;
use cranelift::prelude::AbiParam;
use cranelift::prelude::InstBuilder;
use cranelift::prelude::IntCC;
use cranelift_codegen::ir::{entities::Value, types};
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
            ast::Expression::Comparison(lhs, op, rhs) => {
                let left = self.translate_expr(&*lhs);
                let right = self.translate_expr(&*rhs);
                if left != right {
                    panic!("Cannot compare {left} and {right}");
                };
                match op {
                    ast::ComparisonOperator::Eq => {
                        self.builder.ins().icmp(IntCC::Equal, left, right)
                    }
                    ast::ComparisonOperator::Gt => {
                        self.builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThan, left, right)
                    }
                    ast::ComparisonOperator::Lt => {
                        self.builder.ins().icmp(IntCC::SignedLessThan, left, right)
                    }
                }
            }
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
        let mut sig = self.module.make_signature();
        for arg in &expr.args {
            let arg = self
                .variables
                .get(&arg.ident)
                .expect("Variable not found in scope");
            sig.params.push(AbiParam::new(arg.var_type));
        }
        sig.returns.push(AbiParam::new(types::I32));
        let callee = self
            .module
            .declare_function(&expr.func, Linkage::Import, &sig)
            .unwrap();
        let local_callee = self.module.declare_func_in_func(callee, self.builder.func);

        let mut arg_values = Vec::new();
        for arg in &expr.args {
            let arg_variable = self.variables.get(&arg.ident).unwrap().reference.clone();
            let cloned = arg_variable.clone();
            let arg_value = self.builder.use_var(cloned);
            arg_values.push(arg_value);
        }

        let call = self.builder.ins().call(local_callee, &arg_values);
        let result = self.builder.inst_results(call);
        if result.len() > 0 {
            return result[0];
        } else {
            println!("{:?}", self.builder.func);
            println!("{:?}", arg_values);
            println!("{:?}", result);
            return result[0];
        }
        // let result = self.builder.inst_results(call)[0];
    }

    fn translate_return(&mut self, expr: &ast::Expression) -> Value {
        println!("In translate_return {expr:?}");
        let value = self.translate_expr(expr);
        self.builder.ins().return_(&[value]);
        value
    }

    fn translate_assign(&mut self, expr: &ast::Assignment) -> Value {
        let name = &expr.target.ident;
        let var = self
            .variables
            .get(name)
            .expect(format!("No variable named {}", name).as_str());
        let value = match &expr.value {
            ast::Value::Float(val) => self.builder.ins().f64const(*val as f64),
            ast::Value::Integer(val) => self.builder.ins().iconst(types::I32, *val as i64),
        };
        self.builder.def_var(var.reference, value);
        value
    }
}
