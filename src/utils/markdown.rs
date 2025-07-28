/// Utility functions for handling Telegram MarkdownV2 formatting
/// 
/// MarkdownV2 requires escaping of special characters to prevent formatting issues.
/// This module provides centralized functions for proper text escaping.
/// Escapes markdown special characters for MarkdownV2 parsing mode
/// 
/// This function escapes all characters that have special meaning in Telegram's
/// MarkdownV2 format to ensure they are displayed as literal text.
/// 
/// # Arguments
/// * `text` - The text to escape
/// 
/// # Returns
/// A string with all markdown special characters escaped with backslashes
/// 
/// # Example
/// ```
/// use dnd_scheduler_bot::utils::markdown::escape_markdown;
/// 
/// let text = "Hello *world* (test)";
/// let escaped = escape_markdown(text);
/// assert_eq!(escaped, "Hello \\*world\\* \\(test\\)");
/// ```
pub fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_basic_markdown() {
        assert_eq!(escape_markdown("Hello *world*"), "Hello \\*world\\*");
        assert_eq!(escape_markdown("_italic_"), "\\_italic\\_");
        assert_eq!(escape_markdown("`code`"), "\\`code\\`");
    }

    #[test]
    fn test_escape_brackets_and_parentheses() {
        assert_eq!(escape_markdown("[link](url)"), "\\[link\\]\\(url\\)");
        assert_eq!(escape_markdown("{code}"), "\\{code\\}");
    }

    #[test]
    fn test_escape_special_symbols() {
        assert_eq!(escape_markdown("# Header"), "\\# Header");
        assert_eq!(escape_markdown("- List item"), "\\- List item");
        assert_eq!(escape_markdown("+ Plus sign"), "\\+ Plus sign");
        assert_eq!(escape_markdown("= Equal sign"), "\\= Equal sign");
        assert_eq!(escape_markdown("| Pipe"), "\\| Pipe");
        assert_eq!(escape_markdown("> Quote"), "\\> Quote");
        assert_eq!(escape_markdown("~ Tilde"), "\\~ Tilde");
        assert_eq!(escape_markdown(". Period"), "\\. Period");
        assert_eq!(escape_markdown("! Exclamation"), "\\! Exclamation");
    }

    #[test]
    fn test_escape_empty_and_plain_text() {
        assert_eq!(escape_markdown(""), "");
        assert_eq!(escape_markdown("plain text"), "plain text");
        assert_eq!(escape_markdown("123 ABC"), "123 ABC");
    }

    #[test]
    fn test_escape_complex_text() {
        let input = "Session: *D&D Night* [2024-01-01] (5 players) - Confirmed!";
        let expected = "Session: \\*D&D Night\\* \\[2024\\-01\\-01\\] \\(5 players\\) \\- Confirmed\\!";
        assert_eq!(escape_markdown(input), expected);
    }
}