// Half Sleep -- Mutation Testing for Rust
// Written in 2015 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

use syntax::{ast, fold};
use syntax::ptr::P;

pub struct Mutator;

impl Mutator {
    /// Create a new Mutator
    pub fn new() -> Mutator {
        Mutator
    }
}

impl fold::Folder for Mutator {
    fn fold_expr(&mut self, e: P<ast::Expr>) -> P<ast::Expr> {
        e.map(|e|
            match e.node {
                ast::Expr_::ExprIf(ref expr, ref block, ref elseexpr) => {
                    // Flip the if statement
                    let newexpr = ast::Expr {
                        id: expr.id,
                        node: ast::Expr_::ExprUnary(ast::UnNot, expr.clone()),
                        span: expr.span
                    };
                    let new_e = ast::Expr {
                        id: e.id,
                        node: ast::Expr_::ExprIf (P(newexpr), block.clone(), elseexpr.clone()),
                        span: e.span
                    };
                    // ...and continue
                    fold::noop_fold_expr(new_e, self)
                },
                // At loops we stop recursing because I'm unsure how to guarantee
                // that I will not cause infinite loops when screwing around in
                // there
                ast::Expr_::ExprWhile(_, _, _) |
                ast::Expr_::ExprWhileLet(_, _, _, _) |
                ast::Expr_::ExprForLoop(_, _, _, _) |
                ast::Expr_::ExprLoop(_, _) => {
                    e
                },
                // Everything else just continue folding
                _ => fold::noop_fold_expr(e, self)
            }
        )
    }
}


