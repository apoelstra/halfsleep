
#![feature(custom_attribute, plugin)]

#![plugin(halfsleep)]
#![cfg_attr(test, mutation_test)]

#[mutate]
pub fn four(n: u32) -> u32 {
    if n == 4 {
        4
    } else {
        100
    }
}

#[cfg(test)]
mod tests {
    use super::four;

    #[test]
    fn good_test() {
        assert_eq!(four(4), 4);
        assert_eq!(four(2), 100);
    }

    #[test]
    fn bad_test() {
        assert_eq!(four(4), 4);
    }
}

