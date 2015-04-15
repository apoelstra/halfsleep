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
use syntax::ext::base::{Annotatable, MultiModifier};
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
        MultiModifier(Box::new(expand_mutation_test)));
}

/// emit a warning for #[mutation_test] markers on things which don't use them
fn complain_if_useless_annotation(cx: &mut ExtCtxt, item: &Annotatable) {
    let (warn, sp) = match *item {
        Annotatable::Item(ref i) => {
            (match i.node {
                 ast::Item_::ItemMod(_) | ast::Item_::ItemFn(_, _, _, _, _) => false,
                 _ => true
             }, i.span)
        },
        Annotatable::TraitItem(ref i) => {
            (match i.node {
                 ast::TraitItem_::MethodTraitItem(_, _) => false,
                 _ => true
             }, i.span)
        },
        Annotatable::ImplItem(ref i) => {
            (match i.node {
                 ast::ImplItem_::MethodImplItem(_, _) => false,
                 _ => true
             }, i.span)
        }
    };

    if warn {
        cx.span_warn(sp, "#[mutation_test] has no effect except on modules and functions");
    }
}

/// Create several mutated copies of a given function
pub fn expand_mutation_test(cx: &mut ExtCtxt, _span: Span,
                            _meta: &ast::MetaItem, item: Annotatable)
                           -> Annotatable {

    complain_if_useless_annotation(cx, &item);

    // Check something sane has been decorated
    let this_mod = if let Annotatable::Item(ref ast_item) = item {
        if let ast::ItemMod(ref this_mod) = ast_item.node {
            this_mod
        } else {
            // We do nothing directly for decorated functions; when looking at
            // a module we make a note of which ones are decorated, then consider
            // them all at once. This lets us (a) keep track of how many should-fail
            // test have been created and (b) locate all the unit tests in the
            // first place.
            // TODO: this clone is a lazy way around lack of non-lexically scoped borrows
            return item.clone();
        }
    } else {
        // TODO: this clone is a lazy way around lack of non-lexically scoped borrows
        return item.clone();
    };

    // At this point we know we are looking at a module
    // Locate all the decorated functions
    let item = if let Annotatable::Item(i) = item { i } else { unreachable!() };
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
    // Put the module into an item struct
    let mut item = (*item).clone();
    item.node = ast::Item_::ItemMod(this_mod);

    // Add new unit tests...
    let mut test_duper = test_duper::TestDuper::new(&loc);
    // ...and replace the module in the AST
    Annotatable::Item(P(test_duper.fold_item_simple(item)))
}

