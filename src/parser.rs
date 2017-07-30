use regex::{Regex};

pub fn find_maths_fragments<'a>(text: &'a str) -> Vec<&'a str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(^|[^\$`])`\$([^\$]+)\$`([^\$`]|$)").unwrap();
    }
    RE.captures_iter(text).map(|cap| cap.get(2).unwrap().as_str()).collect()
}

#[cfg(test)]
mod test {
    use parser::*;

    #[test]
    fn none_test() {
        assert!(
            find_maths_fragments(r"The cost of one pineapple is $1.50; the cost of a second is $90; This thing ends with the close delimiter $`")
                == Vec::<&str>::new()
        );
    }

    #[test]
    fn two_in_middle_test() {
        assert!(
            find_maths_fragments(r"Hello! `$3 + 5 = 7$` is one equation. `$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$` is another one.")
                == vec!["3 + 5 = 7", r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"]
        );
    }

    #[test]
    fn start_test() {
        assert!(
            find_maths_fragments(r"`$abc$` was at the start")
                == vec!["abc"]
        );
    }

    #[test]
    fn end_test() {
        assert!(
            find_maths_fragments(r"at the end is `$abc$`")
                == vec!["abc"]
        );
    }

    #[test]
    fn start_and_end_test() {
        assert!(
            find_maths_fragments(r"`$abc$`")
                == vec!["abc"]
        );
    }

    #[test]
    fn test_adjacent() {
        assert!(
            find_maths_fragments(r"`$abc$``$xyz$`")
                == Vec::<&str>::new()
        );

        assert!(
            find_maths_fragments(r"`$lol$`rofl`$kek$`")
                == vec!["lol", "kek"]
        );
    }

    #[test]
    fn sanity_test() {
        /// Tests the equality behaviour I rely on in other tests

        assert!(
            find_maths_fragments(r"Hello! `$3 + 5 = 7$` is one equation. `$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$` is another one.")
                != Vec::<&str>::new()
        );

        assert!(
            find_maths_fragments(r"Hello! `$3 + 5 = 7$` is one equation. `$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$` is another one.")
                != vec!["a", "b"]
        );
    }
}
