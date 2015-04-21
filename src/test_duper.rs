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

use aster;
use std::collections::HashMap;
use std::iter;
use syntax::{ast, attr, codemap, fold};
use syntax::parse::token;
use syntax::ptr::P;
use syntax::util::small_vector::SmallVector;

use locator;
use util;

pub struct TestDuper<'a> {
    /// The locator which knows all the search/replace mappings
    loc: &'a locator::Locator,
    /// A stack which is used to track unit test creation. Basically for
    /// each search/replace pair it finds all unit tests with `search`,
    /// copies them into a new supertest, and replaces `search` with `replace`
    test_stack: Vec<HashMap<(&'a ast::Ident, Vec<ast::PathSegment>), Vec<ast::Item>>>
}

impl<'a> TestDuper<'a> {
    /// Create a new unit test duplicator
    pub fn new(loc: &'a locator::Locator) -> TestDuper<'a> {
        TestDuper {
            loc: loc,
            test_stack: vec![],
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
                        // Run through each mutated fn
                        for (search, replace) in self.loc.name_mappings.iter() {
                            for path in replace.iter().cloned() {
                                let mut replacer = SingleTestDuper::new(*search, &path, self.test_stack.len());
                                let new_copy = replacer.fold_item_simple((*item).clone());
                                // No need to rename since the copies will be in (separate) local scopes
                                if replacer.did_anything {
                                    // Put it into the list of unit tests for this search/replace pair
                                    // This list will contain all unit tests for this pairing (so one
                                    // per each original unit test) and they will all be combined in
                                    // the end. This way we test "each replacement causes at least -one-
                                    // unit test to fail" rather than demanding they all do, which is wrong.
                                    let entry = self.test_stack.last_mut().unwrap().entry((&search, path.clone()));
                                    let tests = entry.or_insert(vec![]);
                                    tests.push(new_copy);
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
                        SmallVector::one(item)
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
            // For modules we need to track depth
            ast::Item_::ItemMod(_) => {
                // Setup a stack frame
                self.test_stack.push(HashMap::new());
                // Recurse to obtain list of tests
                let mut ret = fold::noop_fold_item_simple((*item).clone(), self);
                // Build a new test for each search/replace pair
                for (&(ref search, ref path), test_list) in self.test_stack.last().unwrap().iter() {
                    // Build test function
                    let mut fn_ = aster::AstBuilder::new()
                                      .item()
                                      .attr().word("test")
                                      .attr().word("should_panic")
                                      .fn_(format!("_mutation_test_change_{}_to{}",
                                                   search.name.as_str(),
                                                   path.last().unwrap().identifier.name.as_str()))
                                      .build(ast::FunctionRetTy::DefaultReturn(codemap::DUMMY_SP))
                                      .block();
                    for test in test_list.iter() {
                        fn_ = fn_.stmt().build_item(P(test.clone()));
                        fn_ = fn_.stmt().semi().call().id(test.ident).build();
                    }
                    // Install it
                    if let ast::Item_::ItemMod(ref mut m) = ret.node {
                        m.items.push(fn_.build());
                    } else {
                        unreachable!()
                    }
                }
                // Delete the stack frame
                self.test_stack.pop();
                SmallVector::one(P(ret))
            }
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
struct SingleTestDuper<'a> {
    depth: usize,
    search: ast::Ident,
    replace: &'a [ast::PathSegment],
    did_anything: bool
}

impl<'a> SingleTestDuper<'a> {
    fn new(search: ast::Ident, replace: &'a [ast::PathSegment], depth: usize) -> SingleTestDuper {
        SingleTestDuper {
            depth: depth,
            search: search,
            replace: replace,
            did_anything: false
        }
    }
}

impl<'a> fold::Folder for SingleTestDuper<'a> {
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
            // compute relative path
            let mut segments = vec![util::str_to_pathseg("super"); self.depth];
            // ...(skip the first element of the replace path since that is the common ancestor)
            segments.extend(self.replace.iter().skip(1).cloned());
            ast::Path {
                span: codemap::DUMMY_SP,
                global: false,
                segments: segments
            }
        } else {
            path
        }
    }

    fn fold_tts(&mut self, tts: &[ast::TokenTree]) -> Vec<ast::TokenTree> {
        let mut ret = vec![];
        for tt in tts.iter() {
            match *tt {
                ast::TokenTree::TtToken(span, ref tok) => {
                    if let token::Token::Ident(ref ident, _) = *tok {
                        if ident.name == self.search.name {
                            // mark this SingleTestDuper as successful
                            self.did_anything = true;
                            // build super::super::mod::mod::mod::ident path
                            let mut except_first = false;
                            for seg in iter::repeat(util::str_to_pathseg("super"))
                                           .take(self.depth)
                                           .chain(self.replace.iter().skip(1).cloned()) {
                                if except_first {
                                    ret.push(ast::TokenTree::TtToken(span, token::Token::ModSep));
                                }
                                except_first = true;
                                ret.push(ast::TokenTree::TtToken(span, token::Token::Ident(seg.identifier, token::IdentStyle::ModName)));
                            }
                        } else {
                            ret.push(self.fold_tt(tt))
                        }
                    } else {
                        ret.push(self.fold_tt(tt))
                    }
                }
                _ => ret.push(self.fold_tt(tt))
            }
        }
        ret
    }

    fn fold_mac(&mut self, _mac: ast::Mac) -> ast::Mac {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Folder prior to
        // macro expansion otherwise ("here be dragons")
        fold::noop_fold_mac(_mac, self)
    }
}

 
