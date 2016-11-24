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

#![allow(custom_derive, plugin, plugin_registrar, rustc_private)]

extern crate aster;
extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ext::base::{Annotatable, MultiModifier};
use syntax::parse::token;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::fold::Folder;
use syntax::ptr::P;

mod locator;
mod mutator;
mod test_duper;
mod util;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("mutation_test"),
        MultiModifier(Box::new(expand_mutation_test)));

    reg.register_syntax_extension(
        token::intern("mutate"),
        MultiModifier(Box::new(expand_mutate)));
}

/// check whether an annotation is on a function or function-like object
fn annotation_is_fn(item: &Annotatable) -> bool {
    match *item {
        Annotatable::Item(ref i) => {
            match i.node {
                ast::Item_::ItemMod(_) | ast::Item_::ItemFn(_, _, _, _, _) => true,
                _ => false
            }
        },
        Annotatable::TraitItem(ref i) => {
            match i.node {
                ast::TraitItem_::MethodTraitItem(_, _) => true,
                _ => false
            }
        },
        Annotatable::ImplItem(ref i) => {
            match i.node {
                ast::ImplItem_::MethodImplItem(_, _) => true,
                _ => false
            }
        }
    }
}

/// This is an annotation on functions; we do not use the syntax extender
/// to handle these, rather we do our own AST walk when parsing #[mutation_test]
/// on the module. So all this does is emit warnings for.
pub fn expand_mutate(cx: &mut ExtCtxt, decorator_span: Span,
                     _meta: &ast::MetaItem, item: Annotatable)
                    -> Annotatable {
    if !annotation_is_fn(&item) {
        cx.span_warn(decorator_span, "#[mutate] has no effect except functions and methods");
    }
    item
}

/// This annotation should only be applied to modules
pub fn expand_mutation_test(cx: &mut ExtCtxt, decorator_span: Span,
                            _meta: &ast::MetaItem, item: Annotatable)
                           -> Annotatable {

    // Ensure that we are actually looking at a module
    let this_mod = match item {
        Annotatable::Item(ref ast_item) => {
            if let ast::ItemMod(ref this_mod) = ast_item.node {
                Some(this_mod.clone())
            } else {
                None
            }
        },
        _ => None
    };
    // Is there a less awkward way to do this?
    if this_mod.is_none() {
        if annotation_is_fn(&item) {
            cx.span_warn(decorator_span, concat!("#[mutation_test] can only be applied to modules; to ",
                                                 "mark a function for mutation use #[mutate]"));
        } else {
            cx.span_warn(decorator_span, "#[mutation_test] can only be applied to modules");
        }
        return item;
    };

    // At this point we know we are looking at a module
    // Run through the module duplicating and marring annotated functions
    let item = item.expect_item();
    let mut loc = locator::Locator::new();
    let item = loc.fold_item_simple((*item).clone());

    // Add new unit tests...
    let mut test_duper = test_duper::TestDuper::new(&loc);
    // ...and replace the module in the AST
    Annotatable::Item(P(test_duper.fold_item_simple(item)))
}

