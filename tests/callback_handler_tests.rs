use dnd_scheduler_bot::bot::handlers::callback::escape_markdown;

#[test]
fn test_escape_markdown_basic() {
    assert_eq!(escape_markdown("Hello World"), "Hello World");
    assert_eq!(escape_markdown("Test_underscore"), "Test\\_underscore");
    assert_eq!(escape_markdown("Test*asterisk"), "Test\\*asterisk");
}

#[test]
fn test_escape_markdown_complex() {
    let input = "Test_text*with[special]characters(and)others!";
    let expected = "Test\\_text\\*with\\[special\\]characters\\(and\\)others\\!";
    assert_eq!(escape_markdown(input), expected);
}

#[test]
fn test_escape_markdown_time_format() {
    let input = "Friday, 29 November at 19:30";
    let expected = "Friday, 29 November at 19:30";
    assert_eq!(escape_markdown(input), expected);
}

#[test]
fn test_escape_markdown_special_chars() {
    assert_eq!(escape_markdown("Test.dot"), "Test\\.dot");
    assert_eq!(escape_markdown("Test-dash"), "Test\\-dash");
    assert_eq!(escape_markdown("Test+plus"), "Test\\+plus");
    assert_eq!(escape_markdown("Test=equals"), "Test\\=equals");
    assert_eq!(escape_markdown("Test|pipe"), "Test\\|pipe");
    assert_eq!(escape_markdown("Test{brace}"), "Test\\{brace\\}");
}

#[test]
fn test_escape_markdown_multiple_chars() {
    let input = "Test_*[all]-(special)+chars!";
    let expected = "Test\\_\\*\\[all\\]\\-\\(special\\)\\+chars\\!";
    assert_eq!(escape_markdown(input), expected);
}