use std::{fs::File, ops::Deref};
use std::io::Write;

use koopa::ir::Function;
use koopa::ir::{entities::ValueData, values::Return, FunctionData, Program, ValueKind};

struct AsmBuilder<'a> {
    program: &'a Program,
    current_func: Option<Function>,
    current_bb: Option<koopa::ir::BasicBlock>,
}

impl AsmBuilder<'_> {
    fn current_func_data(&self) -> Option<&FunctionData> {
        self.current_func.map(|func| self.program.func(func))
    }

    fn set_current_func(&mut self, func: Function) {
        self.current_func = Some(func);
    }
}

trait GenerateAsm {
    fn generate(&self, builder: &mut AsmBuilder, file: &mut File) -> std::io::Result<()>;
}

impl GenerateAsm for Return {
    fn generate(&self, builder: &mut AsmBuilder, file: &mut File) -> std::io::Result<()> {
        if let Some(value) = self.value() {
            let value_data = builder.current_func_data().unwrap().dfg().value(value);
            if let ValueKind::Integer(i) = value_data.kind() {
                writeln!(file, "    li a0, {}", i.value())?;
            } else {
                unimplemented!()
            }
        }
        writeln!(file, "    ret")?;
        Ok(())
    }
}

impl GenerateAsm for FunctionData {
    fn generate(&self, builder: &mut AsmBuilder, file: &mut File) -> std::io::Result<()> {
        let name = self.name()[1..].to_string();
        writeln!(file, ".globl {}", name)?;
        writeln!(file, "{}:", name)?;
        for (&bb, node) in self.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                match value_data.kind() {
                    ValueKind::Return(v) => v.generate(builder, file)?,
                    _ => unimplemented!()
                }
            }
        }
        Ok(())
    }
}

pub fn assemble(program: &Program, f: &mut File) -> std::io::Result<()> {
    writeln!(f, ".text")?;
    let mut builder = AsmBuilder {
        program,
        current_func: None,
        current_bb: None,
    };
    for &func in program.func_layout() {
        builder.set_current_func(func);
        program.func(func).generate(&mut builder, f)?;
    }
    Ok(())
}