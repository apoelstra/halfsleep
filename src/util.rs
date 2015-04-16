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

//! Utility functions for use in the library

use syntax::ast;
use syntax::parse::token::str_to_ident;

/// Creates a single PathSegment with the given identifier
pub fn ident_to_pathseg(i: ast::Ident) -> ast::PathSegment {
    ast::PathSegment {
        identifier: i,
        parameters: ast::PathParameters::none()
    }
}

/// Creates a single PathSegment from the given string
pub fn str_to_pathseg(s: &str) -> ast::PathSegment {
    ast::PathSegment {
        identifier: str_to_ident(s),
        parameters: ast::PathParameters::none()
    }
}


