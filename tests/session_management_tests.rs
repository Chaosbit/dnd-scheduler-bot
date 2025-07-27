use dnd_scheduler_bot::bot::commands::list::escape_markdown;

#[test]
fn test_list_escape_markdown_basic() {
    assert_eq!(escape_markdown("Game Night"), "Game Night");
    assert_eq!(escape_markdown("D&D Session"), "D&D Session");
}

#[test]
fn test_list_escape_markdown_special_chars() {
    let input = "Session: Level-up & Loot!";
    let expected = "Session: Level\\-up & Loot\\!";
    assert_eq!(escape_markdown(input), expected);
}

#[test]
fn test_list_escape_markdown_complex() {
    let input = "Test_session*with[special]characters!";
    let expected = "Test\\_session\\*with\\[special\\]characters\\!";
    assert_eq!(escape_markdown(input), expected);
}

#[test]
fn test_list_escape_markdown_datetime() {
    let input = "Friday, 29 November at 19:30";
    let expected = "Friday, 29 November at 19:30";
    assert_eq!(escape_markdown(input), expected);
}

#[test] 
fn test_list_escape_markdown_all_special() {
    let input = "_*[]()~`>#+-=|{}.!";
    let expected = "\\_\\*\\[\\]\\(\\)\\~\\`\\>\\#\\+\\-\\=\\|\\{\\}\\.\\!";
    assert_eq!(escape_markdown(input), expected);
}