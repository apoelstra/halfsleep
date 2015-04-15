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

#![feature(custom_derive, plugin, plugin_registrar, rustc_private)]

extern crate rustc;
extern crate syntax;

use std::mem;
use rustc::plugin::Registry;
use syntax::ext::base::Modifier;
use syntax::parse::token;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::fold::Folder;
use syntax::ptr::P;
use syntax::visit::Visitor;

mod locator;
mod mutator;
mod test_duper;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("mutation_test"),
        Modifier(Box::new(expand_mutation_test)));
}

/// Create several mutated copies of a given function
pub fn expand_mutation_test(cx: &mut ExtCtxt, _span: Span,
                            _meta: &ast::MetaItem, item: P<ast::Item>)
                           -> P<ast::Item> {

    // Check something sane has been decorated
    let this_mod = if let ast::ItemFn(_, _, _, _, _) = item.node {
        // We do nothing directly for decorated functions; when looking at
        // a module we make a note of which ones are decorated, then consider
        // them all at once. This lets us (a) keep track of how many should-fail
        // test have been created and (b) locate all the unit tests in the
        // first place.
        return item;
    } else if let ast::ItemMod(ref this_mod) = item.node {
        this_mod
    } else {
        cx.bug("#[mutation_test] can only be applied to functions and modules")
    };

    // At this point we know we are looking at a module
    // Locate all the decorated functions
    let mut loc = locator::Locator::new();
    loc.visit_item(&*item);

    let mut this_mod = this_mod.clone();
    // Attach mutated functions to the module
    let mut fn_list = vec![];
    mem::swap(&mut loc.mutants, &mut fn_list);
    for mut_fn in fn_list {
        println!("pushing fn");
        this_mod.items.push(P(mut_fn))
    }
    // Expand all macros in the module so that we can see all the function calls
    let this_mod = cx.expander().fold_mod(this_mod);
    // Put the module into an item struct
    let mut item = (*item).clone();
    item.node = ast::Item_::ItemMod(this_mod);

    // Add new unit tests...
    let mut test_duper = test_duper::TestDuper::new(&loc);
    // ...and replace the module in the AST
    P(test_duper.fold_item_simple(item))
}

