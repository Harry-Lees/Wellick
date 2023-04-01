use crate::parser::ast::EmptyType;

use super::ast;
use super::variables;
use super::variables::{to_cranelift_type, Variable};

use cranelift::prelude::AbiParam;
use cranelift::prelude::InstBuilder;
use cranelift::prelude::MemFlags;
use cranelift_codegen::ir::{entities::Value, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;
use std::process;

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

    pub fn translate_if(
        &mut self,
        condition: &ast::Expression,
        if_body: &Vec<ast::Stmt>,
        else_body: &Option<Vec<ast::Stmt>>,
    ) -> Value {
        let then_block = self.builder.create_block();
        let merge_block = self.builder.create_block();
        let cond = self.translate_expr(condition);
        self.builder
            .ins()
            .brif(cond, then_block, &[], merge_block, &[]);
        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);

        let mut then_return: Option<Value> = None;
        for expr in if_body {
            match expr {
                // If a terminator expression, like return, is encountered, we don't
                // need to process the rest of the instructions since they're automatically
                // unreachable code.
                ast::Stmt::Return(stmt) => {
                    then_return = Some(self.translate_return(stmt));
                    break;
                }
                stmt => then_return = Some(self.translate_stmt(stmt)),
            }
        }
        if then_return.is_none() {
            panic!("If statement has no body");
        }

        self.builder.switch_to_block(merge_block);
        then_return.unwrap()
    }

    pub fn translate_stmt(&mut self, stmt: &ast::Stmt) -> Value {
        match stmt {
            ast::Stmt::Assign(expr) => self.translate_assign(expr),
            ast::Stmt::If(condition, body, else_body) => {
                self.translate_if(condition, body, else_body)
            }
            ast::Stmt::Return(expr) => self.translate_return(expr),
            ast::Stmt::Call(expr) => self.translate_call(expr),
            ast::Stmt::ReAssign(expr) => self.translate_reassign(expr),
        }
    }

    pub fn translate_expr(&mut self, expr: &ast::Expression) -> Value {
        match expr {
            ast::Expression::Call(val) => self.translate_call(val),
            ast::Expression::Literal(val) => match val {
                ast::Literal::Float(value) => self
                    .builder
                    .ins()
                    .f32const(value.base10_parse::<f32>().unwrap()),
                ast::Literal::Integer(value) => self
                    .builder
                    .ins()
                    .iconst(types::I32, value.base10_parse::<i64>().unwrap()),
            },
            ast::Expression::Identifier(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");

                match var {
                    Variable::Stack(var) => {
                        self.builder
                            .ins()
                            .stack_load(to_cranelift_type(&var.ty), var.base, 0)
                    }
                    Variable::Register(var) => self.builder.use_var(var.base),
                }
            }
            // Address-Of a value, returns a pointer pointing to the stack slot
            // of the variable.
            ast::Expression::AddressOf(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");

                match var {
                    Variable::Stack(var) => self.builder.ins().stack_addr(
                        self.module.target_config().pointer_type(),
                        var.base,
                        0,
                    ),
                    Variable::Register(_) => panic!("Pointer to register variables unsupported"),
                }
            }
            // Dereference a pointer and return the value at that address.
            ast::Expression::DeRef(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");

                match var {
                    Variable::Stack(var) => {
                        let var_type = match &var.ty {
                            EmptyType::Pointer(ty) => to_cranelift_type(ty),
                            ty => {
                                unimplemented!(
                                    "unsupported operation, dereferencing type {:?}",
                                    ty
                                );
                            }
                        };

                        let stack_ptr = self.builder.ins().stack_load(
                            self.module.target_config().pointer_type(),
                            var.base,
                            0,
                        );

                        self.builder
                            .ins()
                            .load(var_type, MemFlags::new(), stack_ptr, 0)
                    }
                    Variable::Register(_) => {
                        panic!("Unsupported operation, dereferencing register variable");
                    }
                }
            }
        }
    }

    fn translate_call(&mut self, expr: &ast::Call) -> Value {
        let mut sig = self.module.make_signature();
        let mut arg_values = Vec::new();
        for arg in &expr.args {
            let func_arg = self.translate_expr(arg);
            arg_values.push(func_arg);
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

    fn translate_reassign(&mut self, expr: &ast::Local) -> Value {
        let name = &expr.target.ident;
        let value = self.translate_expr(&expr.value);

        let var = self.variables.get(name);
        if var.is_none() {
            println!("Cannot find value `{name}` in this scope");
            process::exit(1);
        }

        match var.unwrap() {
            Variable::Register(var) => {
                if !var.mutable {
                    println!("Cannot mutate immutable variable {name}");
                    process::exit(1);
                }
                self.builder.def_var(var.base, value);
            }
            Variable::Stack(var) => {
                if !var.mutable {
                    println!("Cannot mutate immutable variable {name}");
                    process::exit(1);
                }
                self.builder.ins().stack_store(value, var.base, 0);
            }
        }

        value
    }

    fn translate_assign(&mut self, expr: &ast::Assignment) -> Value {
        let name = &expr.target.ident;

        let var = &self
            .variables
            .get(name)
            .expect(format!("No variable named {}", name).as_str())
            .clone();

        let value = match &expr.value {
            ast::Expression::Literal(literal) => match literal {
                ast::Literal::Float(val) => match var.ty() {
                    ast::EmptyType::Float(ast::FloatType::F32) => self
                        .builder
                        .ins()
                        .f32const(val.base10_parse::<f32>().unwrap()),
                    ast::EmptyType::Float(ast::FloatType::F64) => self
                        .builder
                        .ins()
                        .f64const(val.base10_parse::<f64>().unwrap()),
                    _ => {
                        println!("Cannot convert {:?} to {:?}", val, var.ty());
                        process::exit(1);
                    }
                },
                ast::Literal::Integer(val) => match var.ty() {
                    ast::EmptyType::Integer(ast::IntegerType::I32) => self
                        .builder
                        .ins()
                        .iconst(types::I32, val.base10_parse::<i64>().unwrap()),
                    ast::EmptyType::Integer(ast::IntegerType::I64) => self
                        .builder
                        .ins()
                        .iconst(types::I64, val.base10_parse::<i64>().unwrap()),
                    ast::EmptyType::Integer(ast::IntegerType::PointerSize) => self
                        .builder
                        .ins()
                        .iconst(types::I64, val.base10_parse::<i64>().unwrap()),
                    ast::EmptyType::Pointer(_) => self
                        .builder
                        .ins()
                        .iconst(types::I64, val.base10_parse::<i64>().unwrap()),
                    _ => {
                        println!("Cannot convert {:?} to {:?}", val, var.ty());
                        process::exit(1);
                    }
                },
            },
            value => self.translate_expr(value),
        };

        match var {
            Variable::Register(_) => {
                unimplemented!("unsupported operation, assignment to register variable")
            }
            Variable::Stack(var) => {
                let value_type = self.builder.func.dfg.value_type(value);
                if value_type != to_cranelift_type(&var.ty) {
                    println!(
                        "Cannot convert type from {} to {}",
                        value_type,
                        to_cranelift_type(&var.ty)
                    );
                    process::exit(1);
                };

                self.builder.ins().stack_store(value, var.base, 0);
            }
        }

        value
    }
}
