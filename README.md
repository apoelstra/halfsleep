
### Half Sleep -- Mutation Testing For Rust

Half Sleep is a syntax extension for Rust which provides simple mutation testing.
Mutation testing is a testing methodology in which functions are modified in
simple but breaking ways. If the unit tests still pass, they weren't sufficient.

Rust's syntax extension abilities are powerful enough to do this without the
need for any additional tools. The way it works is as follows: on any modules
marked `#[mutation_test]`, it finds functions marked `#[mutation_test]` and
creates copies of them which are mutated in various ways. Then any unit tests
in the module which call those functions are duplicated to use the mutated
variant and marked `#[should_panic]` under the expectation that they will now
fail. (Any tests already marked `#[should_panic]` are ignored rather than
duplicated.)

### Experimental and Unstable

This library is still quite new and subject to rapid change. Pull requests and
feature suggestions are welcome.

### Compiling

To use halfsleep in your project, add to your `Cargo.toml`
```
[dependencies.halfsleep]
git = "https://github.com/apoelstra/halfsleep.git"

```

### Use

Here is an example project using halfsleep:
```
#![feature(custom_attribute, plugin)]

#![plugin(halfsleep)]
#![cfg_attr(test, mutation_test)]

#[mutation_test]
pub fn keep_fours(n: u32) -> u32 {
    if n == 4 { 4 } else { 100 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_test() {
        assert_eq!(keep_fours(4), 4);
    }
}

```
(TODO:
Notice the `use super::*`; a current limitation of halfsleep that should be
eliminated shortly is that it simply renames function calls to the mutated
variants without respect for module boundaries. In future halfsleep should
provide full paths to the functions it creates.)

