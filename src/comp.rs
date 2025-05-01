use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types::I64, Value};
use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, Signature, UserFuncName};
use cranelift_codegen::verifier::verify_function;
use cranelift_codegen::{isa, settings, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{self, Node};

#[derive(Clone)]
struct Fn {
    id: FuncId,
    sig: Signature,
}

pub struct Compiler {
    module: ObjectModule,
    ctx: Context,
    fn_decls: HashMap<String, Fn>,
}

impl Compiler {
    pub fn new(debug: bool) -> Self {
        let settings = settings::builder();
        let shared_flags = settings::Flags::new(settings);

        let target_isa = isa::lookup(target_lexicon::Triple::host())
            .unwrap()
            .finish(shared_flags)
            .unwrap();

        if debug {
            println!("Compiling for {:?}", target_lexicon::Triple::host());
        }

        let builder = ObjectBuilder::new(
            target_isa,
            "aot_module".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();

        let module = ObjectModule::new(builder);
        let ctx = module.make_context();

        Self {
            module,
            ctx,
            fn_decls: HashMap::new(),
        }
    }

    pub fn declare_functions(&mut self, node: &ast::Node) {
        match node {
            ast::Node::Statements(statements) => {
                for statement in statements {
                    self.declare_functions(&statement)
                }
            }
            ast::Node::IfElse {
                condition,
                if_block: if_body,
                else_block: else_body,
            } => {
                self.declare_functions(&condition);
                self.declare_functions(&if_body);
                self.declare_functions(&else_body);
            }
            ast::Node::Define(_mut, _name, expr) => {
                self.declare_functions(&expr);
            }
            ast::Node::Assign(_name, expr) => {
                self.declare_functions(&expr);
            }
            ast::Node::Expr { lhs, rhs, .. } => {
                self.declare_functions(&lhs);
                self.declare_functions(&rhs);
            }
            ast::Node::FnCall(_name, _args) => {}
            ast::Node::VarRef(_name) => {}
            ast::Node::Number(_num) => {}
            ast::Node::Nada => {}
            ast::Node::Loop {
                var,
                iterable,
                inner,
            } => {
                self.declare_functions(iterable);
                self.declare_functions(inner);
            }
            ast::Node::While { condition, inner } => {
                self.declare_functions(condition);
                self.declare_functions(inner);
            }
            ast::Node::FnDef(name, params, body) => {
                self.declare_functions(body);

                let mut sig = self.module.make_signature();
                sig.returns.push(AbiParam::new(I64));
                for _ in params {
                    sig.params.push(AbiParam::new(I64));
                }

                let fn_name = name.clone().unwrap();
                let id = self
                    .module
                    .declare_function(fn_name.as_str(), Linkage::Export, &sig)
                    .unwrap();

                self.fn_decls.insert(fn_name, Fn { id, sig });
            }
            n => todo!("{:?}", n),
        }
    }

    pub fn translate_fn(
        &mut self,
        name: &Option<String>,
        params: &Vec<String>,
        body: &ast::Node,
        debug: bool,
    ) -> Value {
        let fu = self.fn_decls.get(&name.clone().unwrap()).unwrap().clone();

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut func = Function::with_name_signature(
            UserFuncName::testcase(name.clone().unwrap()),
            fu.sig.clone(),
        );
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.append_block_params_for_function_params(block);

        let mut fnbuilder = CustomFunctionBuilder {
            var_index: 0,
            variables: HashMap::new(),
            builder,
        };

        for name in params {
            let var = fnbuilder.new_var(name);
            fnbuilder.builder.declare_var(var, I64);
            let val = fnbuilder.builder.block_params(block)[0];
            fnbuilder.builder.def_var(var, val);
        }

        let result = self.translate_wbuilder(&mut fnbuilder, &Node::Nada, debug);
        let val = self.translate_wbuilder(&mut fnbuilder, body, debug);
        fnbuilder.builder.ins().return_(&[val]);

        fnbuilder.builder.seal_block(block);
        fnbuilder.builder.finalize();

        let flags = settings::Flags::new(settings::builder());
        let res = verify_function(&func, &flags);
        if let Err(errors) = res {
            panic!("{}", errors);
        }

        if debug {
            println!("{}", func.display());
        }

        self.ctx.func = func;
        self.module.define_function(fu.id, &mut self.ctx).unwrap();
        self.module.clear_context(&mut self.ctx);

        result
    }

    fn translate_wbuilder(
        &mut self,
        fnbuilder: &mut CustomFunctionBuilder,
        node: &ast::Node,
        debug: bool,
    ) -> Value {
        match node {
            ast::Node::Statements(statements) => {
                let mut val = self.translate_wbuilder(fnbuilder, &Node::Nada, debug);
                for statement in statements {
                    val = self.translate_wbuilder(fnbuilder, &statement, debug);
                }
                val
            }
            ast::Node::FnDef(name, params, body) => self.translate_fn(name, params, body, debug),
            ast::Node::FnCall(name, args) => {
                let fu = self.fn_decls.get(name).unwrap();

                let fn_ref = self
                    .module
                    .declare_func_in_func(fu.id, &mut fnbuilder.builder.func);
                let evaled_args: Vec<Value> = args
                    .iter()
                    .map(|arg| self.translate_wbuilder(fnbuilder, arg, debug))
                    .collect();

                let call = fnbuilder.builder.ins().call(fn_ref, &[evaled_args[0]]);
                fnbuilder.builder.inst_results(call)[0]
            }
            ast::Node::IfElse {
                condition,
                if_block: if_body,
                else_block: else_body,
            } => {
                let condition_value = self.translate_wbuilder(fnbuilder, condition, debug);

                let if_block = fnbuilder.builder.create_block();
                let else_block = fnbuilder.builder.create_block();
                let return_block = fnbuilder.builder.create_block();
                fnbuilder.builder.append_block_param(return_block, I64);

                fnbuilder
                    .builder
                    .ins()
                    .brif(condition_value, if_block, &[], else_block, &[]);

                fnbuilder.builder.switch_to_block(if_block);
                fnbuilder.builder.seal_block(if_block);
                let if_return = self.translate_wbuilder(fnbuilder, &if_body, debug);
                fnbuilder.builder.ins().jump(return_block, &[if_return]);

                fnbuilder.builder.switch_to_block(else_block);
                fnbuilder.builder.seal_block(else_block);
                let else_return = self.translate_wbuilder(fnbuilder, &else_body, debug);
                fnbuilder.builder.ins().jump(return_block, &[else_return]);

                fnbuilder.builder.switch_to_block(return_block);
                fnbuilder.builder.seal_block(return_block);
                fnbuilder.builder.block_params(return_block)[0]
            }
            ast::Node::While { condition, inner } => {
                let condition_block = fnbuilder.builder.create_block();
                fnbuilder.builder.append_block_param(condition_block, I64);

                let inner_block = fnbuilder.builder.create_block();

                let return_block = fnbuilder.builder.create_block();
                fnbuilder.builder.append_block_param(return_block, I64);

                let zero = self.translate_wbuilder(fnbuilder, &ast::Node::Nada, debug);
                fnbuilder.builder.ins().jump(condition_block, &[zero]);

                fnbuilder.builder.switch_to_block(condition_block);
                let condition_value = self.translate_wbuilder(fnbuilder, &condition, debug);
                let return_value = fnbuilder.builder.block_params(condition_block)[0];
                fnbuilder.builder.ins().brif(
                    condition_value,
                    inner_block,
                    &[],
                    return_block,
                    &[return_value],
                );

                fnbuilder.builder.switch_to_block(inner_block);
                fnbuilder.builder.seal_block(inner_block);
                let inner_return = self.translate_wbuilder(fnbuilder, &inner, debug);
                fnbuilder
                    .builder
                    .ins()
                    .jump(condition_block, &[inner_return]);

                fnbuilder.builder.seal_block(condition_block);

                fnbuilder.builder.switch_to_block(return_block);
                fnbuilder.builder.seal_block(return_block);
                fnbuilder.builder.block_params(return_block)[0]
            }
            ast::Node::Define(_mut, name, expr) => {
                let var = fnbuilder.new_var(name);
                fnbuilder.builder.declare_var(var, I64);
                let val = self.translate_wbuilder(fnbuilder, expr, debug);
                fnbuilder.builder.def_var(var, val);
                val
            }
            ast::Node::Assign(name, expr) => {
                let var = *fnbuilder.variables.get(name).unwrap();
                let val = self.translate_wbuilder(fnbuilder, expr, debug);
                fnbuilder.builder.def_var(var, val);
                val
            }
            ast::Node::VarRef(name) => {
                let var = fnbuilder.variables.get(name).unwrap();
                fnbuilder.builder.use_var(*var)
            }
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = self.translate_wbuilder(fnbuilder, lhs, debug);
                let rhs = self.translate_wbuilder(fnbuilder, rhs, debug);
                match op {
                    ast::Op::Add => fnbuilder.builder.ins().iadd(lhs, rhs),
                    ast::Op::Sub => fnbuilder.builder.ins().isub(lhs, rhs),
                    ast::Op::Mul => fnbuilder.builder.ins().imul(lhs, rhs),
                    ast::Op::Div => todo!(),
                    ast::Op::Eq => fnbuilder.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                    ast::Op::Neq => fnbuilder.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                    ast::Op::Gt => fnbuilder
                        .builder
                        .ins()
                        .icmp(IntCC::SignedGreaterThan, lhs, rhs),
                    ast::Op::Ge => {
                        fnbuilder
                            .builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs)
                    }
                    ast::Op::Lt => fnbuilder
                        .builder
                        .ins()
                        .icmp(IntCC::SignedLessThan, lhs, rhs),
                    ast::Op::Le => {
                        fnbuilder
                            .builder
                            .ins()
                            .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs)
                    }
                }
            }
            ast::Node::Number(num) => fnbuilder.builder.ins().iconst(I64, *num as i64),
            ast::Node::Nada => fnbuilder.builder.ins().iconst(I64, 0),
            n => todo!("{:?}", n),
        }
    }

    pub fn compile(self, debug: bool) {
        // let mut printf_sig = module.make_signature();
        // printf_sig.params.push(AbiParam::new(I64));
        // printf_sig.returns.push(AbiParam::new(I32));
        // let printf_func = module
        //     .declare_function("printf", Linkage::Import, &printf_sig)
        //     .unwrap();

        // let data_id = module
        //     .declare_data("fmt", Linkage::Local, false, false)
        //     .unwrap();
        // let mut data_ctx = DataDescription::new();
        // data_ctx.define(b"%d\n\0".to_vec().into_boxed_slice());
        // module.define_data(data_id, &data_ctx).unwrap();

        let obj = self.module.finish();
        let bytes = obj.emit().unwrap();
        if !Path::new("build").exists() {
            fs::create_dir("build").unwrap();
        }
        let mut file = File::create("build/out.o").unwrap();
        file.write_all(&bytes).unwrap();

        Command::new("musl-gcc")
            .args(&["-static", "build/out.o", "-o", "build/out"])
            .status()
            .unwrap();
    }
}

struct CustomFunctionBuilder<'a> {
    var_index: usize,
    variables: HashMap<String, Variable>,
    builder: FunctionBuilder<'a>,
}

impl<'a> CustomFunctionBuilder<'a> {
    fn new_var(&mut self, name: &String) -> Variable {
        let var = Variable::new(self.var_index);
        self.var_index += 1;

        self.variables.insert(name.to_string(), var);
        var
    }
}
