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

use std::collections::HashMap;
use syntax::{ast, attr, fold};
use syntax::ext::base::Annotatable;
use syntax::ptr::P;
use syntax::util::small_vector::SmallVector;

use mutator;
use util;

pub struct Locator {
    last_path: Vec<ast::PathSegment>,
    /// A mapping from function names to lists of names of their mutated
    /// variants
    pub name_mappings: HashMap<ast::Ident, Vec<Vec<ast::PathSegment>>>,
}

impl Locator {
    /// Create a new Mutator
    pub fn new() -> Locator {
        Locator {
            last_path: vec![],
            name_mappings: HashMap::new(),
        }
    }
}

impl fold::Folder for Locator {
    fn fold_item(&mut self, item: P<ast::Item>) -> SmallVector<P<ast::Item>> {
        match item.node {
            // If we find a function, record it
            ast::Item_::ItemFn(_, _, _, _, _) => {
                // Is this a function that we want to make mutated copies of?
                if attr::contains_name(&item.attrs, "mutate") {
                    let mut ret = vec![item.clone()];
                    macro_rules! mutate(($mutator:expr, $item:expr) => ({
                        // Build the mutated function
                        let mut_fn = mutator::mutate(&mut $mutator, Annotatable::Item($item.clone()));
                        // Add its rename to the table
                        {
                            // need own scope since we mutably borrow `self`, which
                            // we do again later when calling `noop_fold_item`
                            let entry = self.name_mappings.entry($item.ident);
                            let renames = entry.or_insert(vec![]);
                            let mut path = self.last_path.clone();
                            path.push(util::ident_to_pathseg(mut_fn.ident));
                            renames.push(path);
                        }
                        // Queue it for attachment to AST
                        ret.push(P(fold::noop_fold_item_simple(mut_fn, self)));
                    }));

                    mutate!(mutator::IfSwap::new(), item);
                    mutate!(mutator::IfTrue::new(), item);
                    mutate!(mutator::IfFalse::new(), item);

                    // put all the items on the stack...
                    SmallVector::many(ret)
                } else {
                    // ...otherwise just continue
                    fold::noop_fold_item(item, self)
                }
            },
            // If this is a module, we push its ident onto the pathstack
            // for the "full path" computation
            ast::Item_::ItemMod(_) => {
                self.last_path.push(util::ident_to_pathseg(item.ident));
                let ret = fold::noop_fold_item(item, self);
                self.last_path.pop();
                ret
            }
            _ => {
                // Continue
                fold::noop_fold_item(item, self)
            }
        }
    }

    fn fold_mac(&mut self, _mac: ast::Mac) -> ast::Mac {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Folder prior to
        // macro expansion otherwise ("here be dragons")
        fold::noop_fold_mac(_mac, self)
    }
}

