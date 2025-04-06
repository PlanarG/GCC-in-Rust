use crate::ir::Constructor;

#[derive(Debug)]
pub struct Ast {
    // A program contains a list of global symbols. Each symbol can be either a:
    // - Value declaration
    //     - Constant
    //     - Variable
    // - Function
    pub obj: Vec<GlobalObj>
}

#[derive(Debug)]
pub enum GlobalObj {
    Decl(Decl), 
    Func(Func)
}

// currently, we don't support closures. All functions are global.
#[derive(Debug)]
pub enum Decl {
    Const(Type, Vec<Init>),
    Var(Type, Vec<Init>)
}

#[derive(Debug)]
pub struct Init {
    pub name: String,
    pub value: Option<Expr>
}

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
    pub ty: Type
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Type
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>
}

#[derive(Clone, Copy, Debug)]
pub enum Type {
    Int,
    Void
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr)
}

#[derive(Debug, Clone)]
pub enum Value {
    Num(i32),
    LVal(String)
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value), 
    Binary(MultiOp, Box<Expr>, Box<Expr>), 
    Unary(UnaryOp, Box<Expr>)
}

#[derive(Debug, Clone, Copy)]
pub enum MultiOp {
    Add, 
    Sub,
    Mul,
    Div,
    Mod,
    And, 
    Or,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Not,
    Neg,
    Pos
}

impl From<Type> for koopa::ir::Type {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Int => koopa::ir::Type::get_i32(),
            Type::Void => koopa::ir::Type::get_unit()
        }
    }
}

impl From<Param> for (Option<String>, koopa::ir::Type) {
    fn from(param: Param) -> Self {
        (Some(format!("@{}", param.name)), param.ty.into())
    }
}

impl Expr {
    pub fn is_num(&self) -> bool {
        match self {
            Expr::Value(Value::Num(_)) => true,
            _ => false
        }
    }
    
    pub fn reduce(&mut self, constructor: &Constructor) {
        match self {
            Expr::Value(val) => {
                match val {
                    Value::Num(_) => {},
                    Value::LVal(name) => {
                        if let Some(val) = constructor.get_const_val(name) {
                            *self = Expr::Value(Value::Num(val));
                        }
                    }
                }
            },
            Expr::Binary(op, lhs, rhs) => {
                lhs.reduce(constructor);
                rhs.reduce(constructor);
                if let (Expr::Value(Value::Num(lhs)), Expr::Value(Value::Num(rhs))) = (&**lhs, &**rhs) {
                    *self = Expr::Value(Value::Num(match op {
                        MultiOp::Add => lhs + rhs,
                        MultiOp::Sub => lhs - rhs,
                        MultiOp::Mul => lhs * rhs,
                        MultiOp::Div => lhs / rhs,
                        MultiOp::Mod => lhs % rhs,
                        MultiOp::And => ((*lhs != 0) && (*rhs != 0)) as i32,
                        MultiOp::Or => ((*lhs != 0) || (*rhs != 0)) as i32,   
                        MultiOp::Eq => (lhs == rhs) as i32,
                        MultiOp::Ne => (lhs != rhs) as i32,
                        MultiOp::Lt => (lhs < rhs) as i32,
                        MultiOp::Gt => (lhs > rhs) as i32,
                        MultiOp::Le => (lhs <= rhs) as i32,
                        MultiOp::Ge => (lhs >= rhs) as i32,
                    }));
                }
            },
            Expr::Unary(op, expr) => {
                expr.reduce(constructor);
                if let Expr::Value(Value::Num(expr)) = &**expr {
                    *self = Expr::Value(Value::Num(match op {
                        UnaryOp::Neg => -expr,
                        UnaryOp::Not => !(*expr != 0) as i32,
                        UnaryOp::Pos => *expr
                    }));
                }
            }
        }
    }
}

