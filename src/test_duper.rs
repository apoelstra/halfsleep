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

use syntax::{ast, attr, codemap, fold};
use syntax::parse::token;
use syntax::ptr::P;
use syntax::util::small_vector::SmallVector;

use locator;

pub struct TestDuper<'a> {
    loc: &'a locator::Locator
}

impl<'a> TestDuper<'a> {
    /// Create a new unit test duplicator
    pub fn new(loc: &'a locator::Locator) -> TestDuper<'a> {
        TestDuper {
            loc: loc
        }
    }
}

impl<'a> fold::Folder for TestDuper<'a> {
    fn fold_item(&mut self, item: P<ast::Item>) -> SmallVector<P<ast::Item>> {
        match item.node {
            // Look for unit test functions
            ast::Item_::ItemFn(_, _, _, _, _) => {
                // Is this a unit test?
                if attr::contains_name(&item.attrs, "test") {
                    // Is it a normal (not should_panic) test?
                    if !attr::contains_name(&item.attrs, "should_panic") {
                        // Create copy, put unadulterated copy into the list of unit tests
                        let mut prototype = (*item).clone();
                        let mut copies = vec![P(prototype.clone())];
                        // Make remaining copies should_panic, as they will be mutated
                        prototype.attrs.push(codemap::Spanned {
                            node: ast::Attribute_ {
                                id: attr::mk_attr_id(),
                                style: ast::AttrStyle::AttrOuter,
                                value: P(codemap::Spanned {
                                    node: ast::MetaItem_::MetaWord(token::intern_and_get_ident("should_panic")),
                                    span: codemap::DUMMY_SP
                                }),
                                is_sugared_doc: false
                            },
                            span: codemap::DUMMY_SP
                        });

                        // Run through each mutated fn
                        for (search, replace) in self.loc.name_mappings.iter() {
                            for repl in replace.iter() {
                                let mut replacer = SingleTestDuper::new(*search, *repl);
                                let mut new_copy = replacer.fold_item_simple(prototype.clone());
                                let new_name = format!("_should_panic_{}_mutated_for{}",
                                                       new_copy.ident.name.as_str(),
                                                       repl.name.as_str());
                                new_copy.ident = ast::Ident::new(token::intern(&new_name));
                                if replacer.did_anything {
                                    copies.push(P(new_copy));
                                } else {
                                    // if it did nothing for this search->replace mapping,
                                    // changing the replacement won't make it do something,
                                    // so we can just stop here.
                                    break;
                                }
                            }
                        }

                        // Note that we do not recurse into the unit tests; it appears that
                        // nested unit tests are not run (and who would do this??) so we do
                        // not bother duplicating them.
                        SmallVector::many(copies)
                    } else {
                        // ...nor do we recurse for should_panic tests...
                        SmallVector::one(item)
                    }
                } else {
                    // ...nor do we recurse for non-unit tests; if you nest a #[test]
                    // function inside another function, it is not run regardless of
                    // the testiness of the enclosing function.
                    SmallVector::one(item)
                }
            },
            // Everything else just continue folding
            _ => fold::noop_fold_item(item, self)
        }
    }

    fn fold_mac(&mut self, _mac: ast::Mac) -> ast::Mac {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Folder prior to
        // macro expansion otherwise ("here be dragons")
        fold::noop_fold_mac(_mac, self)
    }
}

/// A SingleTestDuper goes through a function replacing all calls to
/// the function `search` with calls to `replace`
struct SingleTestDuper {
    search: ast::Ident,
    replace: ast::Ident,
    did_anything: bool
}

impl SingleTestDuper {
    fn new(search: ast::Ident, replace: ast::Ident) -> SingleTestDuper {
        SingleTestDuper {
            search: search,
            replace: replace,
            did_anything: false
        }
    }
}

impl fold::Folder for SingleTestDuper {
    fn fold_path(&mut self, path: ast::Path) -> ast::Path {
        // TODO: can we sensibly support segments of length > 1? For that
        //       matter, we are comparing names, which is unhygienic; is
        //       there a way we can do this hygenically at this point in
        //       the parse? Look into this. cf comment near libsyntax/ast.rs:143
        if path.segments.len() == 1 &&
           path.segments[0].identifier.name == self.search.name {
            // mark this SingleTestDuper as successful
            self.did_anything = true;
            // search-and-replace
            let mut new_path = path.clone();
            new_path.segments[0].identifier = self.replace;
            new_path
        } else {
            path
        }
    }

    fn fold_tt(&mut self, tt: &ast::TokenTree) -> ast::TokenTree {
        match *tt {
            ast::TokenTree::TtToken(span, ref tok) => {
                if let token::Token::Ident(ref ident, style) = *tok {
                    if ident.name == self.search.name {
                        // mark this SingleTestDuper as successful
                        self.did_anything = true;
                        // search-and-replace
                        ast::TokenTree::TtToken(span, token::Token::Ident(self.replace, style))
                    } else {
                        fold::noop_fold_tt(tt, self)
                    }
                } else {
                    fold::noop_fold_tt(tt, self)
                }
            }
            _ => fold::noop_fold_tt(tt, self)
        }
    }

    fn fold_mac(&mut self, _mac: ast::Mac) -> ast::Mac {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Folder prior to
        // macro expansion otherwise ("here be dragons")
        fold::noop_fold_mac(_mac, self)
    }
}

 
