//! Run this file with `cargo test --test 02_split_items`.

//! Implement a struct called `SplitItems`, which will receive a string slice and a delimiter
//! char in its constructor.
//!
//! The struct should act as an iterator which iterates over all substrings of the input, separated
//! by the delimiter. The iterator should never return an empty string; it should automatically skip
//! over empty strings.
struct SplitItems<'a> {
    slice: &'a str,
    delimiter: char,
}

impl<'a> SplitItems<'a> {
    fn new(slice: &'a str, delimiter: char) -> Self {
        SplitItems { slice, delimiter }
    }
}

impl<'a> Iterator for SplitItems<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.slice = self.slice.trim_start_matches(self.delimiter);
        if self.slice.is_empty() {
            return None;
        }
        let last = self
            .slice
            .chars()
            .position(|c| c == self.delimiter)
            .unwrap_or(self.slice.len());
        let result = &self.slice[..last];
        self.slice = &self.slice[last..];
        Some(result)
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::SplitItems;

    #[test]
    fn split_empty() {
        let result = SplitItems::new("", ' ').collect::<Vec<_>>();
        assert!(result.is_empty());
    }

    #[test]
    fn split_one_delimiter() {
        let result = SplitItems::new("c", 'c').collect::<Vec<_>>();
        assert!(result.is_empty());
    }

    #[test]
    fn split_only_delimiters() {
        let result = SplitItems::new("ccc", 'c').collect::<Vec<_>>();
        assert!(result.is_empty());
    }

    #[test]
    fn split_delimiter_at_beginning() {
        let result = SplitItems::new(" asd", ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["asd"]);
    }

    #[test]
    fn split_delimiters_at_beginning() {
        let result = SplitItems::new("  asd", ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["asd"]);
    }

    #[test]
    fn split_delimiter_at_end() {
        let result = SplitItems::new("asd ", ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["asd"]);
    }

    #[test]
    fn split_delimiters_at_end() {
        let result = SplitItems::new("asd  ", ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["asd"]);
    }

    #[test]
    fn split_single_chars() {
        let result = SplitItems::new("a b c d e", ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn split_complex() {
        let result = SplitItems::new("   abc   bde casdqw dee xe ", ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["abc", "bde", "casdqw", "dee", "xe"]);
    }

    #[test]
    fn split_complex_custom_delimiter() {
        let result = SplitItems::new("pppabcpppbdepcasdqwpdeepxep", 'p').collect::<Vec<_>>();
        assert_eq!(result, vec!["abc", "bde", "casdqw", "dee", "xe"]);
    }

    #[test]
    fn split_check_reference() {
        let data = String::from("foo bar");
        let result = SplitItems::new(&data, ' ').collect::<Vec<_>>();
        assert_eq!(result, vec!["foo", "bar"]);
    }

    #[test]
    fn split_check_type() {
        let result: SplitItems<'_> = SplitItems::new("foo bar baz", ' ');
        assert_eq!(result.collect::<Vec<_>>(), vec!["foo", "bar", "baz"]);
    }
}
