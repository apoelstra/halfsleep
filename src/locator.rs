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

use syntax::parse::token;
use syntax::{ast, attr, visit};
use syntax::fold::Folder;

use mutator;

pub struct Locator {
    pub mutants: Vec<ast::Item>
}

impl Locator {
    /// Create a new Mutator
    pub fn new() -> Locator {
        Locator {
            mutants: vec![]
        }
    }
}

impl<'a> visit::Visitor<'a> for Locator {
    fn visit_item(&mut self, item: &'a ast::Item) {
        // If we find a function, record it
        if let ast::Item_::ItemFn(_, _, _, _, _) = item.node {
            if attr::contains_name(&item.attrs, "mutation_test") {
                println!("found fn {:}", item.ident.to_string());

                // Build the mutated function
                let mut mutator = mutator::Mutator::new();
                let mut mut_fn = mutator.fold_item_simple(item.clone());
                // Change its name from the original
                mut_fn.ident = ast::Ident::new(token::intern(&format!("mutation_test_copy_of_{}", item.ident.name.as_str())));
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


