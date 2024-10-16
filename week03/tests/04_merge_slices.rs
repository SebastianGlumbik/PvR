//! Run this file with `cargo test --test 04_merge_slices`.

// Implement a function called `merge_slices`, which is useful for the merge sort algorithm.
// It will take two sorted `u32` slices as inputs and merge them into a sorted vector (Vec).
// The function will return the vector.
// Bonus: Can you build a complete merge sort on top of this function? :)

// Used pseudocode from: https://en.wikipedia.org/wiki/Merge_sort#Top-down_implementation_using_lists
fn merge_slices(mut left: &[u32], mut right: &[u32]) -> Vec<u32> {
    let mut result = Vec::<u32>::with_capacity(left.len() + right.len());
    loop {
        match (left.first(), right.first()) {
            (Some(l), r) if r.is_none() || l <= r.unwrap() => {
                result.push(*l);
                left = &left[1..]
            }
            (l, Some(r)) if l.is_none() || l.unwrap() > r => {
                result.push(*r);
                right = &right[1..]
            }
            _ => break,
        }
    }
    result
}

fn mergesort(items: &[u32]) -> Vec<u32> {
    if items.len() <= 1 {
        return items.to_vec();
    }
    let left = mergesort(&items[0..(items.len() / 2)]);
    let right = mergesort(&items[(items.len() / 2)..]);
    merge_slices(left.as_slice(), right.as_slice())
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::{merge_slices, mergesort};

    #[test]
    fn merge_slices_empty() {
        assert_eq!(merge_slices(&[], &[]), vec![]);
    }

    #[test]
    fn merge_slices_basic() {
        assert_eq!(merge_slices(&[1, 2, 3], &[4, 5, 6]), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn merge_slices_interleaved() {
        assert_eq!(merge_slices(&[1, 3, 5], &[2, 4, 6]), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn merge_slices_duplicates() {
        assert_eq!(merge_slices(&[1, 1, 3], &[1, 3, 4]), vec![1, 1, 1, 3, 3, 4]);
    }

    #[test]
    fn merge_slices_uneven_size() {
        assert_eq!(
            merge_slices(&[1, 4, 6, 8], &[0, 1, 1, 3, 4, 5, 7, 8, 9]),
            vec![0, 1, 1, 1, 3, 4, 4, 5, 6, 7, 8, 8, 9]
        );
    }

    #[test]
    fn merge_slices_first_empty() {
        assert_eq!(merge_slices(&[], &[1, 4, 8]), vec![1, 4, 8]);
    }

    #[test]
    fn merge_slices_second_empty() {
        assert_eq!(merge_slices(&[1, 9, 11], &[]), vec![1, 9, 11]);
    }

    // Mergesort tests
    #[test]
    fn mergesort_empty() {
        assert_eq!(mergesort(&[]), vec![]);
    }
    #[test]
    fn mergesort_one() {
        assert_eq!(mergesort(&[1]), vec![1]);
    }
    #[test]
    fn mergesort_multiple_1() {
        assert_eq!(mergesort(&[1, 5, 4, 3, 2]), vec![1, 2, 3, 4, 5]);
    }
    #[test]
    fn mergesort_multiple_2() {
        assert_eq!(
            mergesort(&[6, 7, 1, 3, 2, 4, 8, 5]),
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );
    }
}
