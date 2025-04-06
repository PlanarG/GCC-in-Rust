
use std::collections::HashMap;

use crate::ast::*;
use koopa::ir::{builder::{BasicBlockBuilder, LocalBuilder, LocalInstBuilder, ValueBuilder}, layout::BasicBlockNode, BasicBlock, Function, FunctionData, Program};
use koopa::ir::BinaryOp;

pub struct Constructor {
    program: Program,
    current_func: Option<Function>,
    current_bb: Option<BasicBlock>, 
    const_symbols: HashMap<String, i32>,
    var_symbols: HashMap<String, Option<koopa::ir::Value>>,
}

impl Constructor {
    fn new() -> Self {
        Self { 
            program: Program::new(), 
            current_func: None,
            current_bb: None, 
            const_symbols: HashMap::new(),
            var_symbols: HashMap::new(),
        }
    }

    pub fn get_const_val(&self, name: &str) -> Option<i32> {
        self.const_symbols.get(name).cloned()
    }

    fn new_func(&mut self, func: &Func) {
        let func = self.program.new_func(FunctionData::with_param_names(
            format!("@{}", func.name), 
            func.params.iter().map(|x| x.clone().into()).collect(), 
            func.ty.clone().into()
        ));
        self.current_func = Some(func);
        self.current_bb = None;
    }

    fn new_bb(&mut self, name: String) {
        let func_data = self.get_func_data().unwrap();
        let bb = func_data.dfg_mut().new_bb().basic_block(Some(name));
        func_data.layout_mut().bbs_mut().push_key_back(bb).unwrap();
        self.current_bb = Some(bb);
    }

    fn get_bb(&mut self) -> Option<&mut BasicBlockNode> {
        let bb = self.current_bb.clone()?;
        let func = self.get_func_data()?;
        Some(func.layout_mut().bb_mut(bb))
    }

    fn get_func_data(&mut self) -> Option<&mut FunctionData> {
        self.current_func.map(|func| self.program.func_mut(func))
    }

    fn new_value(&mut self) -> LocalBuilder{
        self.get_func_data().unwrap().dfg_mut().new_value()
    }

}

trait Construct {
    type Output;
    fn construct(&self, constructor: &mut Constructor) -> Self::Output; 
}

impl Construct for GlobalObj {
    type Output = ();

    fn construct(&self, constructor: &mut Constructor) {
        match self {
            GlobalObj::Decl(decl) => decl.construct(constructor),
            GlobalObj::Func(func) => func.construct(constructor),
        };
    }
}

impl Construct for Decl {
    type Output = ();

    fn construct(&self, constructor: &mut Constructor) {
        match self {
            Decl::Var(_, inits) => {
                for init in inits {
                    if let Some(value) = &init.value {
                        let mut value_reduced = value.clone();
                        value_reduced.reduce(constructor);
                        let result = value_reduced.construct(constructor);
                        constructor.var_symbols.insert(init.name.clone(), Some(result));
                    } else {
                        constructor.var_symbols.insert(init.name.clone(), None);
                    }
                } 
            }
            Decl::Const(_, inits) => {
                for init in inits {
                    let mut value_reduced = init.value.as_ref().unwrap().clone();
                    value_reduced.reduce(constructor);

                    if let Expr::Value(Value::Num(num)) = value_reduced {
                        constructor.const_symbols.insert(init.name.clone(), num);
                    } else {
                        unreachable!();
                    }
                }
            }
        }

    }
}

impl Construct for Func {
    type Output = ();
    fn construct(&self, constructor: &mut Constructor) {
        constructor.new_func(self);
        constructor.new_bb("%entry".to_string());
        self.body.construct(constructor);
    }
}

impl Construct for Block {
    type Output = ();

    fn construct(&self, constructor: &mut Constructor) {
        for stmt in &self.stmts {
            stmt.construct(constructor);
        }
    }
}

impl Construct for Stmt {
    type Output = ();

    fn construct(&self, constructor: &mut Constructor) {
        match self {
            Stmt::Return(expr) => {
                let mut expr_reduce = expr.clone();
                expr_reduce.reduce(&constructor);
                let res = expr_reduce.construct(constructor);
                let ret = constructor.new_value().ret(Some(res));
                constructor.get_bb().unwrap().insts_mut().push_key_back(ret).unwrap();
            }
        }
    }
}

impl Construct for Expr {
    type Output = koopa::ir::Value;

    fn construct(&self, constructor: &mut Constructor) -> Self::Output {
        macro_rules! push_binary {
            ($op:ident, $l_res:expr, $r_res:expr) => {{
                let var = constructor.new_value().binary(BinaryOp::$op, $l_res, $r_res);
                constructor.get_bb().unwrap().insts_mut().push_key_back(var).unwrap();
                var
            }}
        }

        match self {
            Expr::Value(value) => value.construct(constructor),
            Expr::Binary(op, lhs, rhs) => {
                let l_res = lhs.construct(constructor);
                let r_res = rhs.construct(constructor);

                match op {
                    MultiOp::Add => push_binary!(Add, l_res, r_res),
                    MultiOp::Sub => push_binary!(Sub, l_res, r_res),
                    MultiOp::Mul => push_binary!(Mul, l_res, r_res),
                    MultiOp::Div => push_binary!(Div, l_res, r_res),
                    MultiOp::Mod => push_binary!(Mod, l_res, r_res),
                    MultiOp::And => todo!(),
                    MultiOp::Or => todo!(),
                    MultiOp::Eq => todo!(),
                    MultiOp::Ne => todo!(),
                    MultiOp::Lt => todo!(),
                    MultiOp::Gt => todo!(),
                    MultiOp::Le => todo!(),
                    MultiOp::Ge => todo!(),
                }
            },
            Expr::Unary(op, expr) => {
                let res = expr.construct(constructor);
                let zero = constructor.new_value().integer(0);
                match op {
                    UnaryOp::Neg => push_binary!(Sub, zero, res),
                    UnaryOp::Not => push_binary!(Eq, zero, res),
                    UnaryOp::Pos => res,
                }
            },
        }
    }
}

impl Construct for Value {
    type Output = koopa::ir::Value;

    fn construct(&self, constructor: &mut Constructor) -> Self::Output {
        match self {
            Value::Num(num) => constructor.new_value().integer(*num),
            Value::LVal(name) => {
                todo!()
            }
        }
    }
}

pub fn construct(ast: &Ast) -> Program {
    let mut constructor = Constructor::new();
    for obj in &ast.obj {
        obj.construct(&mut constructor);
    }
    constructor.program
}