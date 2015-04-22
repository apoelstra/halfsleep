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
    fn test_one() {
        assert_eq!(keep_fours(4), 4);
    }

    #[test]
    fn test_two() {
        assert_eq!(keep_fours(2), 100);
    }

    #[test]
    #[should_panic]
    fn test_ignore_me() {
        assert_eq!(keep_fours(2), 50);
    }
}

