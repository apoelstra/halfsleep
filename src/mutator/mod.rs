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
use syntax::ext::base::Annotatable;
use syntax::parse::token;

pub mod if_swap;
pub mod if_true;
pub mod if_false;

pub use self::if_swap::IfSwap;
pub use self::if_true::IfTrue;
pub use self::if_false::IfFalse;

/// An object which is able to mutate functions passed into it, e.g.
/// by replacing all the if statements with their negations
pub trait Mutator: fold::Folder {
    fn rename(&self, old_name: &str) -> String;
}

/// Use a mutator to produce a function
pub fn mutate<M: Mutator>(mutator: &mut M, item: Annotatable) -> ast::Item {
    match item {
        Annotatable::Item(item) => {
            // Obtain changed name
            let new_name = mutator.rename(item.ident.name.as_str());
            // Mutate the function
            let mut mut_fn = mutator.fold_item_simple((*item).clone());
            // Insert changed name
            mut_fn.ident = ast::Ident::new(token::intern(&new_name));
            // Return
            mut_fn
        },
        _ => unimplemented!()
    }
}



