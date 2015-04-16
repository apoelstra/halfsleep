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
use syntax::{ast, attr, visit};
use syntax::ext::base::Annotatable;
use syntax::ptr::P;

use mutator;
use util;

pub struct Locator {
    last_path: Vec<ast::PathSegment>,
    /// A list of mutated functions that need to be inserted into the AST
    pub mutants: Vec<ast::Item>,
    /// A mapping from function names to lists of names of their mutated
    /// variants
    pub name_mappings: HashMap<ast::Ident, Vec<Vec<ast::PathSegment>>>,
}

impl Locator {
    /// Create a new Mutator
    pub fn new() -> Locator {
        Locator {
            last_path: vec![],
            mutants: vec![],
            name_mappings: HashMap::new(),
        }
    }
}

impl<'a> visit::Visitor<'a> for Locator {
    fn visit_item(&mut self, item: &'a ast::Item) {
        match item.node {
            // If we find a function, record it
            ast::Item_::ItemFn(_, _, _, _, _) => {
                // Is this a function that we want to make mutated copies of?
                if attr::contains_name(&item.attrs, "mutate") {
                    macro_rules! mutate(($mutator:expr, $item:expr) => ({
                        // Build the mutated function
                        let mut_fn = mutator::mutate(&mut $mutator, Annotatable::Item(P($item.clone())));
                        // Add its rename to the table
                        let entry = self.name_mappings.entry($item.ident);
                        let renames = entry.or_insert(vec![]);
                        let mut path = self.last_path.clone();
                        path.push(util::ident_to_pathseg(mut_fn.ident));
                        renames.push(path);
                        // Queue it for attachment to AST
                        self.mutants.push(mut_fn);
                    }));

                    mutate!(mutator::IfSwap::new(), item);
                    mutate!(mutator::IfTrue::new(), item);
                    mutate!(mutator::IfFalse::new(), item);

                    // Continue visiting
                    visit::walk_item(self, item);
                }
            },
            // If this is a module, we push its ident onto the pathstack
            ast::Item_::ItemMod(_) => {
                self.last_path.push(util::ident_to_pathseg(item.ident));
                visit::walk_item(self, item);
                self.last_path.pop();
            }
            _ => {
                // Continue visiting
                visit::walk_item(self, item);
            }
        }
    }

    fn visit_mac(&mut self, _mac: &'a ast::Mac) {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Visitor prior to
        // macro expansion otherwise ("here be dragons")
    }
}

