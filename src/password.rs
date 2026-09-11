use rand::Rng;

const ALNUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const SYMBOLS: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// Generates a random password of `length` characters, drawn from letters
/// and digits, plus punctuation unless `no_symbols` is set.
pub(crate) fn generate_password(length: usize, no_symbols: bool) -> String {
    let charset: Vec<u8> = if no_symbols {
        ALNUM.to_vec()
    } else {
        ALNUM.iter().chain(SYMBOLS).copied().collect()
    };
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| charset[rng.gen_range(0..charset.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_password_has_requested_length() {
        assert_eq!(generate_password(16, false).len(), 16);
        assert_eq!(generate_password(0, false).len(), 0);
    }

    #[test]
    fn generate_password_no_symbols_is_alphanumeric_only() {
        let password = generate_password(200, true);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn generate_password_with_symbols_can_include_symbols() {
        let found_symbol = (0..50).any(|_| {
            generate_password(64, false)
                .chars()
                .any(|c| !c.is_ascii_alphanumeric())
        });
        assert!(found_symbol, "expected at least one symbol across 50 tries");
    }

    #[test]
    fn generate_password_is_random() {
        assert_ne!(generate_password(32, false), generate_password(32, false));
    }
}
