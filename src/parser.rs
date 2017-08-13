use regex::{Regex};

#[derive(Debug, PartialEq, Eq)]
pub enum MessageType {
    Plain,
    LaTeX
}

pub fn classify_message(text: &str) -> MessageType {
    lazy_static! {
        static ref LATEX_RE: Regex = Regex::new(r"\$.*\$($|[^0-9])|\\begin").unwrap();
    }
    if LATEX_RE.is_match(text) {
        MessageType::LaTeX
    } else {
        MessageType::Plain
    }
}

#[cfg(test)]
mod test {
    use parser::*;

    #[test]
    fn simple_plain_test() {
        assert!(
            classify_message(r"Hello, how are you?")
                == MessageType::Plain
        );
    }

    #[test]
    fn dollars_plain_test() {
        assert!(
            classify_message(r"The cost of one pineapple is $1.50; the cost of a second is $90")
                == MessageType::Plain
        );
    }

    #[test]
    fn which_is_pretty_cool_test() {
        assert!(
            classify_message(r"The square root of x is denoted $\sqrt{x}$, which is pretty cool")
                == MessageType::LaTeX
        );
    }


    #[test]
    fn two_in_middle_test() {
        assert!(
            classify_message(r"Hello! $3 + 5 = 7$ is one equation. $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$ is another one.")
                == MessageType::LaTeX
        );
    }

    #[test]
    fn start_test() {
        assert!(
            classify_message(r"$abc$ was at the start")
                == MessageType::LaTeX
        );
    }

    #[test]
    fn end_test() {
        assert!(
            classify_message(r"at the end is $abc$")
                == MessageType::LaTeX
        );
    }

    #[test]
    fn start_and_end_test() {
        assert!(
            classify_message(r"$abc$")
                == MessageType::LaTeX
        );
    }

    #[test]
    fn environment_test() {
        assert!(
            classify_message(r"\begin{center} abc \end{center}")
                == MessageType::LaTeX
        );
    }

    #[test]
    fn environment_surrounded_test() {
        assert!(
            classify_message(r"will be centered; \begin{center} abc \end{center}; was centered.")
                == MessageType::LaTeX
        );
    }
}
