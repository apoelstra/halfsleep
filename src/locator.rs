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
use syntax::parse::token;
use syntax::{ast, attr, visit};
use syntax::fold::Folder;

use mutator;

pub struct Locator {
    /// A list of mutated functions that need to be inserted into the AST
    pub mutants: Vec<ast::Item>,
    /// A mapping from function names to lists of names of their mutated
    /// variants
    pub name_mappings: HashMap<ast::Ident, Vec<ast::Ident>>,
}

impl Locator {
    /// Create a new Mutator
    pub fn new() -> Locator {
        Locator {
            mutants: vec![],
            name_mappings: HashMap::new(),
        }
    }
}

impl<'a> visit::Visitor<'a> for Locator {
    fn visit_item(&mut self, item: &'a ast::Item) {
        // If we find a function, record it
        if let ast::Item_::ItemFn(_, _, _, _, _) = item.node {
            // Is this a function that we want to make mutated copies of?
            if attr::contains_name(&item.attrs, "mutation_test") {
                // Build the mutated function
                let mut mutator = mutator::Mutator::new();
                let mut mut_fn = mutator.fold_item_simple(item.clone());

                // Change its name from the original
                let new_name = format!("_mutated_{}", item.ident.name.as_str());
                mut_fn.ident = ast::Ident::new(token::intern(&new_name));
                let entry = self.name_mappings.entry(item.ident);
                let renames = entry.or_insert(vec![]);
                renames.push(mut_fn.ident);

                // Queue it for attachment to AST
                self.mutants.push(mut_fn);
            }
        }

        // Continue visiting
        visit::walk_item(self, item);
    }

    fn visit_mac(&mut self, _mac: &'a ast::Mac) {
        // do nothing -- we have to implement this though because the
        // compiler will yell at us about using a Visitor prior to
        // macro expansion otherwise ("here be dragons")
    }
}
