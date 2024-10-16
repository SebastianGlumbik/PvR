//! Run this file with `cargo test --test 03_interleave`.

// Implement a function called `interleave`, which will take two string slices and return
// a string that contains the characters from the two input strings interleaved.
// The first character of the output should start with the first character of the first argument.
// See tests for details.
//
// Can you write the function without any explicit indexing (`str[index]`)?
//
// Hint: you can use `string.chars()` to create an iterator over the Unicode characters of a string.

fn interleave<'a>(slice1: &'a str, slice2: &'a str) -> String {
    let mut iter1 = slice1.chars().peekable();
    let mut iter2 = slice2.chars().peekable();
    let mut output = String::with_capacity(slice1.len() + slice2.len());
    loop {
        if let Some(c) = iter1.next() {
            output.push(c)
        }
        if let Some(c) = iter2.next() {
            output.push(c)
        }

        if iter1.peek().is_none() && iter2.peek().is_none() {
            break output;
        }
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::interleave;

    #[test]
    fn interleave_empty() {
        assert_eq!(interleave("", ""), "");
    }

    #[test]
    fn interleave_only_left() {
        assert_eq!(interleave("a", ""), "a");
        assert_eq!(interleave("zxjas", ""), "zxjas");
    }

    #[test]
    fn interleave_only_right() {
        assert_eq!(interleave("", "z"), "z");
        assert_eq!(interleave("", "foobar"), "foobar");
    }

    #[test]
    fn interleave_same_length() {
        assert_eq!(interleave("abcdef", "012345"), "a0b1c2d3e4f5");
    }

    #[test]
    fn interleave_first_longer() {
        assert_eq!(
            interleave("Programming Rust", "O'Reilly"),
            "POr'oRgerialmlmying Rust"
        );
    }

    #[test]
    fn interleave_second_longer() {
        assert_eq!(
            interleave("ahoj, jak se máš?", "díky za optání, mám se dobře"),
            "adhíokjy,  zjaa ko psteá nmíá,š ?mám se dobře"
        );
    }
}
