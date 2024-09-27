//! Run this file with `cargo test --test 04_luhn_algorithm`.

// Implement the Luhn algorithm (https://en.wikipedia.org/wiki/Luhn_algorithm),
// which is used to check the validity of e.g. bank or credit card numbers.
fn luhn_algorithm(mut payload: u64) -> bool {
    if payload < 10 {
        return true;
    }

    let check_digit = payload % 10;
    payload /= 10;

    let mut sum = 0;
    let mut double = true;
    while payload > 0 {
        let mut number = (payload % 10) * if double { 2 } else { 1 };
        number = (number % 10) + number / 10;
        payload /= 10;
        double = !double;
        sum += number;
    }

    (10 - (sum % 10)) == check_digit
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use super::luhn_algorithm;

    #[test]
    fn luhn_zero() {
        assert!(luhn_algorithm(0));
    }

    #[test]
    fn luhn_small_correct() {
        assert!(luhn_algorithm(5));
        assert!(luhn_algorithm(18));
    }

    #[test]
    fn luhn_small_incorrect() {
        assert!(!luhn_algorithm(10));
    }

    #[test]
    fn luhn_correct() {
        assert!(luhn_algorithm(17893729974));
        assert!(luhn_algorithm(79927398713));
    }

    #[test]
    fn luhn_incorrect() {
        assert!(!luhn_algorithm(17893729975));
        assert!(!luhn_algorithm(17893729976));
        assert!(!luhn_algorithm(17893729977));
        assert!(!luhn_algorithm(123456));
    }
}
