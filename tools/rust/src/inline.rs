use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

static RE_BOLD_STAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*").unwrap());
static RE_BOLD_UNDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"__(.+?)__").unwrap());
static RE_ITALIC: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)").unwrap());
static RE_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());
static RE_STRIKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"~~(.+?)~~").unwrap());

static RE_IMG_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap());
static RE_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap());

pub fn strip_inline_markdown(text: &str) -> String {
    let text = RE_BOLD_STAR.replace_all(text, "$1").to_string();
    let text = RE_BOLD_UNDER.replace_all(&text, "$1").to_string();
    let text = RE_ITALIC.replace_all(&text, "$1").to_string();
    let text = RE_CODE.replace_all(&text, "$1").to_string();
    RE_STRIKE.replace_all(&text, "$1").to_string()
}

pub fn process_links(text: &str, keep_urls: bool) -> String {
    if keep_urls {
        let text = RE_IMG_LINK.replace_all(text, "$1<$2>").to_string();
        RE_LINK.replace_all(&text, "$1<$2>").to_string()
    } else {
        let text = RE_IMG_LINK.replace_all(text, "$1").to_string();
        RE_LINK.replace_all(&text, "$1").to_string()
    }
}

pub fn process_inline(text: &str, compression: i32, keep_urls: bool) -> String {
    let text = strip_inline_markdown(text);
    process_links(&text, compression < 2 || keep_urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_stripping() {
        assert_eq!(strip_inline_markdown("**bold**"), "bold");
        assert_eq!(strip_inline_markdown("__bold__"), "bold");
    }

    #[test]
    fn test_italic_stripping() {
        assert_eq!(strip_inline_markdown("*italic*"), "italic");
    }

    #[test]
    fn test_code_stripping() {
        assert_eq!(strip_inline_markdown("`code`"), "code");
    }

    #[test]
    fn test_strikethrough_stripping() {
        assert_eq!(strip_inline_markdown("~~strike~~"), "strike");
    }

    #[test]
    fn test_links_keep_urls() {
        assert_eq!(
            process_links("[text](http://url)", true),
            "text<http://url>"
        );
        assert_eq!(
            process_links("![alt](img.png)", true),
            "alt<img.png>"
        );
    }

    #[test]
    fn test_links_strip_urls() {
        assert_eq!(process_links("[text](http://url)", false), "text");
        assert_eq!(process_links("![alt](img.png)", false), "alt");
    }

    #[test]
    fn test_process_inline_c0() {
        // c0: keep URLs
        assert_eq!(
            process_inline("**bold** [link](url)", 0, false),
            "bold link<url>"
        );
    }

    #[test]
    fn test_process_inline_c2() {
        // c2: strip URLs unless keep_urls
        assert_eq!(
            process_inline("**bold** [link](url)", 2, false),
            "bold link"
        );
    }

    #[test]
    fn test_process_inline_c2_keep_urls() {
        assert_eq!(
            process_inline("**bold** [link](url)", 2, true),
            "bold link<url>"
        );
    }
}
