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
use syntax::parse::token;

pub mod if_swap;

pub use self::if_swap::IfSwap;

/// An object which is able to mutate functions passed into it, e.g.
/// by replacing all the if statements with their negations
pub trait Mutator: fold::Folder {
    fn rename(&self, old_name: &str) -> String;
}

/// Use a mutator to produce a function
pub fn mutate<M: Mutator>(mutator: &mut M, item: ast::Item) -> ast::Item {
    // Obtain changed name
    let new_name = mutator.rename(item.ident.name.as_str());
    // Mutate the function
    let mut mut_fn = mutator.fold_item_simple(item);
    // Insert changed name
    mut_fn.ident = ast::Ident::new(token::intern(&new_name));
    // Return
    mut_fn
}



