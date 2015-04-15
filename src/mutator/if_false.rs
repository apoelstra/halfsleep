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

/// A Mutator which duplicates the else clauses of `if` statements into their
/// bodies, effectively making them `if false`. We do it like this
/// rather than just removing the conditional and promoting the if body
/// to its parent because the conditional may have side effects.
pub struct IfFalse;

impl IfFalse {
    /// Create a new IfFalse
    pub fn new() -> IfFalse {
        IfFalse
    }
}

impl Mutator for IfFalse {
    fn rename(&self, name: &str) -> String {
        format!("_mutate_iffalse_{}", name)
    }
}

impl fold::Folder for IfFalse {
    fn fold_expr(&mut self, e: P<ast::Expr>) -> P<ast::Expr> {
        e.map(|e|
            match e.node {
                ast::Expr_::ExprIf(expr, _, elseexpr) => {
                    // Build the new body
                    let block = ast::Block {
                          stmts: vec![],
                          expr: elseexpr.clone(),
                          id: ast::DUMMY_NODE_ID,
                          rules: ast::BlockCheckMode::DefaultBlock,
                          span: expr.span
                    };
                    // Modify the if statement
                    let new_if = ast::Expr {
                        id: ast::DUMMY_NODE_ID,
                        span: block.span,
                        node: ast::Expr_::ExprIf(expr, P(block), elseexpr)
                    };
                    // ...and continue
                    fold::noop_fold_expr(new_if, self)
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


