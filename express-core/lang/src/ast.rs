#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Ident(String),
    Number(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Factorial,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Const(Literal),
    Function {
        name: Literal,
        args: Vec<Expression>,
    },
    BinOp(Box<Expression>, Box<Expression>, Operation),
    UnOp(Operation, Box<Expression>),
}

// pub enum Optimizable<T> {
//     Opt(T),
//     NonOpt,
// }
//
// /// Provides a Visitor pattern interface to the Expression
// trait Visit {
//     fn visit_const(&mut self, c: &Expression) -> Literal;
//     fn visit_fn(&mut self, f: &Expression) -> Literal;
//     fn visit_binop(&mut self, bop: &Expression) -> Literal;
//     fn visit_unop(&mut self, unop: &Expression) -> Expression;
// }
//
// impl Visit for Expression {
//     fn visit_const(&mut self, c: &Expression) -> Literal {
//         if let Expression::Const(lit) = *c {
//             lit
//         } else {
//             unreachable!()
//         }
//     }
//
//     fn visit_fn(&mut self, f: &Expression) -> (String, Vec<Expression>) {
//         if let Expression::Function { name: n, args: a } = *f {
//             (n, a)
//         }
//     }
//
//     fn visit_binop(&mut self, bop: &Expression) -> f64 {
//         todo!()
//     }
//
//     fn visit_unop(&mut self, unop: &Expression) -> f64 {
//         todo!()
//     }
// }
//
// /// Simplifies AST expression
// pub fn optimize(expr: Optimizable<Expression>) -> Optimizable<Expression> {
//     if let Opt(e) = expr {
//         match e {
//             Expression::BinOp(lhs, rhs, op) => if matches!(optimize(lhs), Some(val)) {},
//             Expression::UnOp(op, rhs) => {}
//             Expression::Const(lit) => match lit {
//                 Literal::Ident(name) => unimplemented!(),
//                 Literal::Number(num) => Optimizable::Opt(num),
//             },
//             _ => Optimizable::NonOpt,
//         }
//     }
// }
