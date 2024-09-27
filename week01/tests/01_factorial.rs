//! Run this file with `cargo test --test 01_factorial`.

// Implement a simple factorial function.
fn factorial(n: u32) -> u32 {
    let mut result = 1;
    for i in 1..=n {
        result *= i;
    }

    result
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use super::factorial;

    #[test]
    fn factorial_0() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn factorial_1() {
        assert_eq!(factorial(1), 1);
    }

    #[test]
    fn factorial_2() {
        assert_eq!(factorial(2), 2);
    }

    #[test]
    fn factorial_5() {
        assert_eq!(factorial(5), 120);
    }
}
