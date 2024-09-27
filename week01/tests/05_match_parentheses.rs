//! Run this file with `cargo test --test 05_match_parentheses`.

// Implement a function called `match_parentheses`.
// It will receive a string slice (`&str`) containing arbitrary characters.
//
// Check that all parentheses within the string (`()`, `[]`, `{}`) are matched properly:
// - Each `(` has to be before `)`, `[` before `]` and `{` before `}`
// - There is an even number of opening and closing parentheses of each kind
// - Parentheses are not mismatched: e.g. `(` cannot be "closed" by `]`
//
// If everything is matched properly, return `true`, otherwise return `false`.
//
// Try to leverage pattern matching.
// Hint: there is a useful basic data structure that can be used for checking parentheses equality.
// It rhymes with "Jack" :)
// You don't even need to implement it fully, just use Vec and perform two operations on it.

use std::ops::Not;

fn match_parentheses(slice: &str) -> bool {
    let mut stack = Vec::new();
    for char in slice.chars() {
        match char {
            '(' | '{' | '[' => {
                stack.push(char);
                continue;
            }
            ')' => {
                if stack.last().is_some_and(|x| *x == '(').not() {
                    return false;
                }
            }
            '}' => {
                if stack.last().is_some_and(|x| *x == '{').not() {
                    return false;
                }
            }
            ']' => {
                if stack.last().is_some_and(|x| *x == '[').not() {
                    return false;
                }
            }
            _ => continue,
        }
        stack.pop();
    }

    stack.is_empty()
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use super::match_parentheses;

    #[test]
    fn match_parentheses_empty() {
        assert!(match_parentheses(""));
        assert!(match_parentheses("foobar"));
    }

    #[test]
    fn match_parentheses_wrong_order() {
        assert!(!match_parentheses(")("));
        assert!(!match_parentheses("xx)y(aa"));
    }

    #[test]
    fn match_parentheses_extra_opening() {
        assert!(!match_parentheses("("));
        assert!(!match_parentheses("x[qq]e(s"));
        assert!(!match_parentheses("[(]"));
        assert!(!match_parentheses("(xxú[ú]😊"));
    }

    #[test]
    fn match_parentheses_extra_closing() {
        assert!(!match_parentheses(")"));
        assert!(!match_parentheses("[])"));
        assert!(!match_parentheses("[)]"));
        assert!(!match_parentheses("x([{)}])y"));
    }

    #[test]
    fn match_parentheses_wrong_matched_type() {
        assert!(!match_parentheses("[)"));
        assert!(!match_parentheses("[qp)"));
        assert!(!match_parentheses("[}xx"));
        assert!(!match_parentheses("p{]vr"));
        assert!(!match_parentheses("y[q}xx"));
        assert!(!match_parentheses("y[qq)x"));
        assert!(!match_parentheses("([})"));
        assert!(!match_parentheses("(((([}))))"));
    }

    #[test]
    fn simple() {
        assert!(match_parentheses("[]"));
        assert!(match_parentheses("()"));
        assert!(match_parentheses("{}"));
        assert!(match_parentheses("{}{}"));
        assert!(match_parentheses("{}{}{}"));
        assert!(match_parentheses("{}[]()"));
        assert!(match_parentheses("y{x}qq(xywe)[][xx]yy[][p]()"));
    }

    #[test]
    fn nested() {
        assert!(match_parentheses("({[]})"));
        assert!(match_parentheses("q(x{x[xqp]yy}y)"));
        assert!(match_parentheses("((((()))))"));
        assert!(match_parentheses("{[({})](([]))}"));
    }
}