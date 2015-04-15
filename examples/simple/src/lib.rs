
#![feature(custom_attribute, plugin)]

#![plugin(halfsleep)]
#![mutation_test]

#[mutation_test]
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
    use super::mutation_test_copy_of_four;

    #[test]
    fn good_test() {
        assert_eq!(four(4), 4);
        assert_eq!(four(2), 100);
    }

    #[test]
    fn bad_test() {
        assert_eq!(four(4), 4);
    }

    #[test]
    fn m_bad_test() {
        assert_eq!(mutation_test_copy_of_four(4), 4);
    }

    #[test]
    fn m_good_test() {
        assert_eq!(mutation_test_copy_of_four(4), 4);
        assert_eq!(mutation_test_copy_of_four(2), 100);
    }
}

