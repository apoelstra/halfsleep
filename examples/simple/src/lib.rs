#![feature(custom_attribute, plugin)]

#![plugin(halfsleep)]
#![cfg_attr(test, mutation_test)]

#[mutate]
pub fn keep_fours(n: u32) -> u32 {
    if n == 4 { 4 } else { 100 }
}

#[cfg(test)]
mod tests {
    use super::keep_fours;

    #[test]
    fn bad_test() {
        assert_eq!(keep_fours(4), 4);
    }
}

