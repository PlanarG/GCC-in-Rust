mod ast;
mod ir;
mod asm;

use koopa::back::KoopaGenerator;
use lalrpop_util::lalrpop_mod;
use clap::Parser;

lalrpop_mod!(sysy);  

#[derive(Parser, Debug)]
struct Args {
    file: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long)]
    koopa: bool,
    #[arg(short, long)]
    riscv: bool
}

fn main() { 
    let mut args: Vec<String> = std::env::args().collect();

    let alternatives = vec![
        ("-koopa", "--koopa"), 
        ("-riscv", "--riscv")
    ];

    for arg in &mut args {
        for (a, b) in &alternatives {
            if arg == a {
                *arg = b.to_string();
            }
        }
    }

    let args = Args::parse_from(args);
    let file = std::fs::read_to_string(&args.file).unwrap();
    let ast = sysy::ProgramParser::new().parse(&file).unwrap();

    let output = args.output.clone();

    let koopa_ir = ir::construct(&ast);
    
    if args.koopa {
        KoopaGenerator::from_path(output.clone()).unwrap().generate_on(&koopa_ir).unwrap();
    } 
    
    if args.riscv {
        let mut file = std::fs::File::create(output).unwrap();
        asm::assemble(&koopa_ir, &mut file).unwrap();
    }
} 