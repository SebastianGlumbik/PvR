//! Run this file with `cargo test --test 02_case_insensitive_cmp`.

//! Implement a struct `CaseInsensitive`, which will allow comparing (=, <, >, etc.)
//! two (ASCII) string slices in a case insensitive way, without performing any reallocations
//! and without modifying the original strings.

use std::cmp::Ordering;

struct CaseInsensitive<'a>(&'a str);

impl<'a> AsRef<str> for CaseInsensitive<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a, T> PartialEq<T> for CaseInsensitive<'a>
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        let mut it_a = self.0.chars();
        let mut it_b = other.as_ref().chars();
        loop {
            let Some(a) = it_a.next() else { break };
            let Some(b) = it_b.next() else { break };
            if a.to_ascii_lowercase() != b.to_ascii_lowercase() {
                return false;
            }
        }

        self.0.len() == other.as_ref().len()
    }
}

impl<'a, T> PartialOrd<T> for CaseInsensitive<'a>
where
    T: AsRef<str>,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        match self.0.len().cmp(&other.as_ref().len()) {
            Ordering::Less => return Some(Ordering::Less),
            Ordering::Greater => return Some(Ordering::Greater),
            Ordering::Equal => (),
        }

        let mut it_a = self.0.chars();
        let mut it_b = other.as_ref().chars();

        loop {
            let Some(a) = it_a.next() else { break };
            let Some(b) = it_b.next() else { break };
            match a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()) {
                Ordering::Less => return Some(Ordering::Less),
                Ordering::Greater => return Some(Ordering::Greater),
                Ordering::Equal => (),
            }
        }

        Some(Ordering::Equal)
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::CaseInsensitive;

    #[test]
    fn case_insensitive_same() {
        assert!(CaseInsensitive("") == CaseInsensitive(""));
        assert!(CaseInsensitive("a") == CaseInsensitive("A"));
        assert!(CaseInsensitive("a") == CaseInsensitive("a"));
        assert!(CaseInsensitive("Foo") == CaseInsensitive(&String::from("fOo")));
        assert!(
            CaseInsensitive("12ABBBcLPQusdaweliAS2") == CaseInsensitive("12AbbbclpQUSdawelias2")
        );
    }

    #[test]
    fn case_insensitive_smaller() {
        assert!(CaseInsensitive("") < CaseInsensitive("a"));
        assert!(CaseInsensitive("a") < CaseInsensitive("B"));
        assert!(CaseInsensitive("aZa") < CaseInsensitive("Zac"));
        assert!(CaseInsensitive("aZ") < CaseInsensitive("Zac"));
        assert!(CaseInsensitive("PWEasUDsx") < CaseInsensitive("PWEaszDsx"));
        assert!(CaseInsensitive("PWEasuDsx") < CaseInsensitive("PWEasZDsx"));
    }

    #[test]
    fn case_insensitive_larger() {
        assert!(CaseInsensitive("a") > CaseInsensitive(""));
        assert!(CaseInsensitive("B") > CaseInsensitive("a"));
        assert!(CaseInsensitive("Zac") > CaseInsensitive("aZa"));
        assert!(CaseInsensitive("Zac") > CaseInsensitive("aZ"));
        assert!(CaseInsensitive("PWEaszDsx") > CaseInsensitive("PWEasUDsx"));
        assert!(CaseInsensitive("PWEasZDsx") > CaseInsensitive("PWEasuDsx"));
    }
}
