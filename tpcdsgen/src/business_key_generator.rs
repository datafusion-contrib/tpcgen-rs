const BUSINESS_KEY_CHARS: &str = "ABCDEFGHIJKLMNOP";

pub fn make_business_key(primary: u64) -> String {
    let key_part1 = long_to_8_char_string((primary >> 32) as u32);
    let key_part2 = long_to_8_char_string(primary as u32);
    format!("{}{}", key_part1, key_part2)
}

fn long_to_8_char_string(mut value: u32) -> String {
    let mut result = String::with_capacity(8);
    for _ in 0..8 {
        let char_index = (value & 0xF) as usize;
        result.push(BUSINESS_KEY_CHARS.chars().nth(char_index).unwrap());
        value >>= 4;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_key_generation() {
        assert_eq!(make_business_key(1), "AAAAAAAABAAAAAAA");
        assert_eq!(make_business_key(2), "AAAAAAAACAAAAAAA");
        assert_eq!(make_business_key(3), "AAAAAAAADAAAAAAA");
    }

    #[test]
    fn test_long_to_8_char_string() {
        assert_eq!(long_to_8_char_string(0), "AAAAAAAA");
        assert_eq!(long_to_8_char_string(1), "BAAAAAAA");
        assert_eq!(long_to_8_char_string(15), "PAAAAAAA");
        assert_eq!(long_to_8_char_string(16), "ABAAAAAA");
    }
}
