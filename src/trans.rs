use std::collections::HashMap;

use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types::*, Value};
use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, Signature, UserFuncName};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

use crate::ast::{self, Node};

pub struct Translator {}

impl Translator {
    pub fn translate(node: &ast::Node) -> Vec<Function> {
        match node {
            ast::Node::Statements(statements) => statements
                .into_iter()
                .map(Self::translate)
                .flatten()
                .collect::<Vec<Function>>(),
            ast::Node::FnDef(name, params, body) => vec![Self::translate_fn(name, params, body)],
            _ => todo!(),
        }
    }

    fn translate_fn(
        name: &Option<String>,
        params: &Vec<String>,
        body: &Box<ast::Node>,
    ) -> Function {
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(I64));
        for _ in params {
            sig.params.push(AbiParam::new(I64));
        }

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut func =
            Function::with_name_signature(UserFuncName::testcase(name.clone().unwrap()), sig);
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.append_block_params_for_function_params(block);

        let mut translator = FunctionTranslator {
            var_index: 0,
            variables: HashMap::new(),
            builder,
        };

        for name in params {
            let var = translator.new_var(name);
            translator.builder.declare_var(var, I64);
            let val = translator.builder.block_params(block)[0];
            translator.builder.def_var(var, val);
        }

        let val = translator.translate(body);
        translator.builder.ins().return_(&[val]);

        translator.builder.seal_block(block);
        translator.builder.finalize();

        let flags = settings::Flags::new(settings::builder());
        let res = verify_function(&func, &flags);
        if let Err(errors) = res {
            panic!("{}", errors);
        }

        func
    }
}

struct FunctionTranslator<'a> {
    var_index: usize,
    variables: HashMap<String, Variable>,
    builder: FunctionBuilder<'a>,
}

impl<'a> FunctionTranslator<'a> {
    fn new_var(&mut self, name: &String) -> Variable {
        let var = Variable::new(self.var_index);
        self.var_index += 1;

        self.variables.insert(name.to_string(), var);
        var
    }

    fn translate(&mut self, expr: &ast::Node) -> Value {
        match expr {
            ast::Node::Statements(statements) => {
                let mut val = self.translate(&Node::Nada);
                for statement in statements {
                    val = self.translate(&statement)
                }
                val
            }
            ast::Node::IfElse {
                condition,
                if_block: if_body,
                else_block: else_body,
            } => {
                let condition_value = self.translate(condition);

                let if_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let return_block = self.builder.create_block();

                self.builder.append_block_param(return_block, I64);

                self.builder
                    .ins()
                    .brif(condition_value, if_block, &[], else_block, &[]);

                self.builder.switch_to_block(if_block);
                self.builder.seal_block(if_block);
                let if_return = self.translate(&if_body);
                self.builder.ins().jump(return_block, &[if_return]);

                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                let else_return = self.translate(&else_body);
                self.builder.ins().jump(return_block, &[else_return]);

                self.builder.switch_to_block(return_block);
                self.builder.seal_block(return_block);
                self.builder.block_params(return_block)[0]
            }
            ast::Node::Define(_mut, name, expr) => {
                let var = self.new_var(name);
                self.builder.declare_var(var, I64);
                let val = self.translate(expr);
                self.builder.def_var(var, val);
                val
            }
            ast::Node::Assign(name, expr) => {
                let var = *self.variables.get(name).unwrap();
                let val = self.translate(expr);
                self.builder.def_var(var, val);
                val
            }
            ast::Node::VarRef(name) => {
                let var = self.variables.get(name).unwrap();
                self.builder.use_var(*var)
            }
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = self.translate(lhs);
                let rhs = self.translate(rhs);
                match op {
                    ast::Op::Add => self.builder.ins().iadd(lhs, rhs),
                    ast::Op::Sub => self.builder.ins().isub(lhs, rhs),
                    ast::Op::Mul => self.builder.ins().imul(lhs, rhs),
                    ast::Op::Div => todo!(),
                    ast::Op::Eq => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                    ast::Op::Neq => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                    ast::Op::Gt => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                    ast::Op::Ge => {
                        self.builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs)
                    }
                    ast::Op::Lt => self
                        .builder
                        .ins()
                        .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                    ast::Op::Le => self
                        .builder
                        .ins()
                        .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                }
            }
            ast::Node::Number(num) => self.builder.ins().iconst(I64, *num as i64),
            ast::Node::Nada => self.builder.ins().iconst(I64, 0),
            n => todo!("{:?}", n),
        }
    }
}
