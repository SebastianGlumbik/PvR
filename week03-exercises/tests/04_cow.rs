//! Run this file with `cargo test --test 04_cow`.

// Implement a function called `to_upper_if_needed`, which takes a string slice
// and returns the uppercase version of that string.
// If the string was already uppercase, it should not perform any allocations!

enum OwnedOrBorrowed<'a> {
    Owned(String),
    Borrowed(&'a str),
}

fn to_upper_if_needed(data: &str) -> OwnedOrBorrowed {
    if data.chars().all(|c| c.is_uppercase()) {
        OwnedOrBorrowed::Borrowed(data)
    } else {
        OwnedOrBorrowed::Owned(data.to_uppercase())
    }
}
