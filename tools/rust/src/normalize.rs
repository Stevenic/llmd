use unicode_normalization::UnicodeNormalization;

pub fn stage0(text: &str) -> Vec<String> {
    let text: String = text.nfkc().collect();
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    text.split('\n')
        .map(|l| l.trim_end().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crlf_normalization() {
        let result = stage0("hello\r\nworld");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_cr_normalization() {
        let result = stage0("hello\rworld");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_trailing_whitespace() {
        let result = stage0("hello   \nworld  ");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_empty_input() {
        let result = stage0("");
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_nfkc_normalization() {
        // \u{FB01} (fi ligature) should be normalized to "fi"
        let result = stage0("\u{FB01}");
        assert_eq!(result, vec!["fi"]);
    }
}
