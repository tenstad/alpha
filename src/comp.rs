use cranelift_codegen::ir::Function;
use cranelift_codegen::{isa, settings};
use cranelift_module::{FuncId, Linkage, Module, ModuleResult};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn compile(funcs: Vec<Function>) {
    let settings = settings::builder();
    let shared_flags = settings::Flags::new(settings);

    let target_isa = isa::lookup(target_lexicon::Triple::host())
        .unwrap()
        .finish(shared_flags)
        .unwrap();

    let builder = ObjectBuilder::new(
        target_isa,
        "aot_module".to_string(),
        cranelift_module::default_libcall_names(),
    )
    .unwrap();

    let mut module = ObjectModule::new(builder);
    let mut ctx = module.make_context();
    funcs
        .into_iter()
        .map(|f| {
            module
                .declare_function(&f.name.to_string()[1..], Linkage::Export, &f.signature)
                .map(|id| (f, id))
        })
        .collect::<ModuleResult<Vec<(Function, FuncId)>>>()
        .unwrap()
        .into_iter()
        .map(|(f, id)| {
            ctx.func = f;
            module
                .define_function(id, &mut ctx)
                .and_then(|_| Ok(module.clear_context(&mut ctx)))
        })
        .collect::<ModuleResult<Vec<()>>>()
        .unwrap();

    let obj = module.finish();
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
