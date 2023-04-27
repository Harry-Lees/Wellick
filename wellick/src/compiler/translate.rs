use crate::parser::ast::EmptyType;

use super::ast;
use super::variables;
use super::variables::{to_cranelift_type, Variable};

use cranelift::prelude::AbiParam;
use cranelift::prelude::InstBuilder;
use cranelift::prelude::MemFlags;
use cranelift::prelude::Signature;
use cranelift_codegen::ir::{entities::Value, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;
use std::process;

/// Module to translate AST into Cranelift IR constructs.
pub struct FunctionTranslator<'a, 'b: 'a> {
    functions: &'a HashMap<String, ast::FnDecl>,
    pub(crate) builder: FunctionBuilder<'b>,
    pub(crate) variables: HashMap<String, variables::Variable>,
    pub(crate) module: &'a mut ObjectModule,
}

impl<'a, 'b> FunctionTranslator<'a, 'b> {
    pub fn new(
        functions: &'a HashMap<String, ast::FnDecl>,
        builder: FunctionBuilder<'b>,
        variables: HashMap<String, variables::Variable>,
        module: &'b mut ObjectModule,
    ) -> Self {
        Self {
            functions,
            builder,
            variables,
            module,
        }
    }

    pub fn translate_if(&mut self, condition: &ast::Expression, if_body: &Vec<ast::Stmt>) -> Value {
        let then_block = self.builder.create_block();
        let merge_block = self.builder.create_block();
        let cond = self.translate_expr(condition);
        self.builder
            .ins()
            .brif(cond, then_block, &[], merge_block, &[]);
        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);

        for stmt in if_body {
            match stmt {
                // If a terminator expression, like return, is encountered, we don't
                // need to process the rest of the instructions since they're automatically
                // unreachable code.
                ast::Stmt::Return(_) => {
                    self.translate_stmt(&stmt);
                    self.builder.switch_to_block(merge_block);
                    self.builder.seal_block(merge_block);
                    return self.builder.ins().iconst(types::I32, 0);
                }
                _ => {
                    self.translate_stmt(&stmt);
                }
            }
        }

        self.builder.ins().jump(merge_block, &[]);
        self.builder.switch_to_block(merge_block);
        self.builder.seal_block(merge_block);
        self.builder.ins().iconst(types::I32, 0)
    }

    pub fn translate_stmt(&mut self, stmt: &ast::Stmt) -> Value {
        match stmt {
            ast::Stmt::Assign(expr) => self.translate_assign(expr),
            ast::Stmt::If(condition, body) => self.translate_if(condition, body),
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
                    .get(&value.name)
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
                            EmptyType::Pointer(ty) => to_cranelift_type(&ty.ty),
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
        let expected_sig = match expr.func.as_str() {
            "iadd" | "isub" | "idiv" | "imul" | "ieq" | "ilteq" | "ilt" | "imod" => Signature {
                params: vec![AbiParam::new(types::I32), AbiParam::new(types::I32)],
                returns: vec![AbiParam::new(types::I32)],
                call_conv: self.module.isa().default_call_conv(),
            },
            "println" | "print" => Signature {
                params: vec![AbiParam::new(types::I32)],
                returns: vec![AbiParam::new(types::I32)],
                call_conv: self.module.isa().default_call_conv(),
            },
            "print_addr" => Signature {
                params: vec![AbiParam::new(types::I64)],
                returns: vec![AbiParam::new(types::I64)],
                call_conv: self.module.isa().default_call_conv(),
            },
            _ => {
                let func = self
                    .functions
                    .get(&expr.func)
                    .expect(format!("function {} not found", expr.func).as_str());

                for (i, param) in expr.args.iter().enumerate() {
                    match param {
                        ast::Expression::AddressOf(param_expr) => {
                            let arg_mutable = match &func.args.get(i).unwrap().t {
                                ast::EmptyType::Pointer(ptr) => ptr.mutable,
                                _ => continue,
                            };

                            if param_expr.mutable && !arg_mutable {
                                println!(
                                    "Expected &{}, got &mut {}",
                                    param_expr.name, param_expr.name
                                );
                                process::exit(1);
                            }

                            if arg_mutable && !param_expr.mutable {
                                println!(
                                    "Expected &mut {}, got &{}",
                                    param_expr.name, param_expr.name
                                );
                                process::exit(1);
                            }
                        }
                        _ => {}
                    }
                }

                Signature {
                    params: func
                        .args
                        .iter()
                        .map(|arg| AbiParam::new(to_cranelift_type(&arg.t)))
                        .collect(),
                    returns: vec![AbiParam::new(to_cranelift_type(&func.ret_type))],
                    call_conv: self.module.isa().default_call_conv(),
                }
            }
        };

        let (params, arg_values): (Vec<AbiParam>, Vec<Value>) = expr
            .args
            .iter()
            .map(|arg| {
                let func_arg = self.translate_expr(arg);
                let arg_type = self.builder.func.dfg.value_type(func_arg);
                (AbiParam::new(arg_type), func_arg)
            })
            .unzip();

        let sig = Signature {
            params,
            returns: expected_sig.returns.clone(),
            call_conv: self.module.isa().default_call_conv(),
        };

        if sig != expected_sig {
            println!(
                "Mismatched types for fn \"{}\", expected \"{}\", got \"{}\"",
                expr.func, expected_sig, sig
            );
            process::exit(1);
        }

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
            ast::Expression::AddressOf(addr_of) => {
                // Check that the mutability of the pointer matches the mutability of the data.

                let assign_target = &self
                    .variables
                    .get(&addr_of.name)
                    .expect(format!("No variable named {}", &addr_of.name).as_str())
                    .clone();

                let var_mutable = match assign_target {
                    Variable::Stack(stack_var) => stack_var.mutable,
                    Variable::Register(reg_var) => reg_var.mutable,
                };

                if let ast::EmptyType::Pointer(ptr) = &expr.var_type {
                    if !var_mutable && ptr.mutable {
                        println!("Cannot declare mutable pointer to {}, as it has not been declared mutable", &addr_of.name);
                        process::exit(1);
                    }
                }
                self.translate_expr(&expr.value)
            }
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
