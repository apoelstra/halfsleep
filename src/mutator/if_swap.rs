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

use mutator::Mutator;

pub struct IfSwap;

impl IfSwap {
    /// Create a new IfSwap
    pub fn new() -> IfSwap {
        IfSwap
    }
}

impl Mutator for IfSwap {
    fn rename(&self, name: &str) -> String {
        format!("_mutate_ifswap_{}", name)
    }
}

impl fold::Folder for IfSwap {
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

    fn fold_mac(&mut self, _mac: ast::Mac) -> ast::Mac {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Folder prior to
        // macro expansion otherwise ("here be dragons")
        fold::noop_fold_mac(_mac, self)
    }
}


