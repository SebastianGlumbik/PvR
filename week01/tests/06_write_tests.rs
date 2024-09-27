//! Run this file with `cargo test --test 06_write_tests`.

/// This function implements a string sanitization logic that should uphold the following
/// properties:
/// - After sanitization, the result must not end with the character `x`
/// - After sanitization, the result must not end with the character `o`
/// - After sanitization, the result must not end with the string `.exe`
///
/// The function assumes that the input to the function only consists of lower and uppercase
/// characters from the English alphabet and digits 0-9.
///
/// The implementation contains some bugs.
///
/// Your task is to write a set (at least 8) of unit tests, use them to find (at least 2) bugs in
/// this function and then fix the function.
fn sanitize(input: &str) -> &str {
    let mut sanitized = input;
    loop {
        let start = sanitized.len();
        // Remove all x from the end of the string
        sanitized = sanitized.trim_end_matches("x");

        // Remove all o from the end of the string
        sanitized = sanitized.trim_end_matches("o");

        // Remove all .exe from the end of the string
        sanitized = sanitized.trim_end_matches(".exe");

        if start == sanitized.len() {
            return sanitized;
        }
    }
}

/// write tests for the `sanitize` function
///
/// Bonus: can you find any bugs using the [proptest](https://proptest-rs.github.io/proptest/intro.html)
/// crate?
/// Note that you will probably need to run `cargo test` with the `PROPTEST_DISABLE_FAILURE_PERSISTENCE=1`
/// environment variable to make proptest work for tests stored in the `tests` directory.
#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn empty() {
        assert_eq!(sanitize(""), "");
        assert_eq!(sanitize("          "), "          ");
    }

    #[test]
    fn without() {
        assert_eq!(sanitize("abc"), "abc");
        assert_eq!(sanitize("ABC"), "ABC");
        assert_eq!(sanitize("123456"), "123456");
        assert_eq!(sanitize("exe"), "exe");
    }

    #[test]
    fn not_at_the_end() {
        assert_eq!(sanitize("xa"), "xa");
        assert_eq!(sanitize("oa"), "oa");
        assert_eq!(sanitize(".exea"), ".exea");
    }

    #[test]
    fn at_the_end_once() {
        assert_eq!(sanitize("x"), "");
        assert_eq!(sanitize("ax"), "a");
        assert_eq!(sanitize("o"), "");
        assert_eq!(sanitize("ao"), "a");
        assert_eq!(sanitize(".exe"), "");
        assert_eq!(sanitize("a.exe"), "a");
    }

    #[test]
    fn upper_case() {
        assert_eq!(sanitize("X"), "X");
        assert_eq!(sanitize("O"), "O");
        assert_eq!(sanitize(".EXE"), ".EXE");
        assert_eq!(sanitize(".Exe"), ".Exe");
    }

    // First bug
    #[test]
    fn multiple_times() {
        assert_eq!(sanitize("xxx"), "");
        assert_eq!(sanitize("xax"), "xa");
        assert_eq!(sanitize("ooo"), "");
        assert_eq!(sanitize("oao"), "oa");
        assert_eq!(sanitize(".exe.exe.exe"), "");
        assert_eq!(sanitize(".exea.exe"), ".exea");
    }

    // Second bug
    #[test]
    fn combined_once() {
        assert_eq!(sanitize(".exeox"), "");
        assert_eq!(sanitize(".exexo"), "");
        assert_eq!(sanitize("ox.exe"), "");
    }

    // Third bug
    #[test]
    fn combined_multiple_times() {
        assert_eq!(sanitize("xox.exe"), "");
        assert_eq!(sanitize("xxo"), "");
        assert_eq!(sanitize("o.exexo.exex"), "");
    }
}
