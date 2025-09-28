use meshbbs::bbs::public::{PublicCommandParser, PublicCommand};

#[test]
fn test_help_command() {
    let parser = PublicCommandParser::new();
    match parser.parse("^help") { PublicCommand::Help => {}, other => panic!("Expected Help, got {:?}", other) }
}

#[test]
fn test_login_command() {
    let parser = PublicCommandParser::new();
    match parser.parse("^login Alice") { PublicCommand::Login(u) => assert_eq!(u, "Alice"), other => panic!("Expected Login, got {:?}", other) }
}

#[test]
fn test_invalid_login_no_name() {
    let parser = PublicCommandParser::new();
    match parser.parse("^login") { PublicCommand::Invalid(_) => {}, other => panic!("Expected Invalid, got {:?}", other) }
}

#[test]
fn test_unknown() {
    let parser = PublicCommandParser::new();
    match parser.parse("garbage") { PublicCommand::Unknown => {}, other => panic!("Expected Unknown, got {:?}", other) }
}

#[test]
fn test_missing_caret_prefix() {
    let parser = PublicCommandParser::new();
    match parser.parse("LOGIN Bob") { PublicCommand::Unknown => {}, other => panic!("Expected Unknown (no caret), got {:?}", other) }
}

#[test]
fn test_weather_command() {
    let parser = PublicCommandParser::new();
    match parser.parse("^WEATHER") { PublicCommand::Weather => {}, other => panic!("Expected Weather, got {:?}", other) }
}

#[test]
fn test_weather_with_args() {
    let parser = PublicCommandParser::new();
    match parser.parse("^WEATHER Portland OR") { PublicCommand::Weather => {}, other => panic!("Expected Weather with args accepted, got {:?}", other) }
}

#[test]
fn test_weather_suffix_not_match() {
    let parser = PublicCommandParser::new();
    match parser.parse("^WEATHERS") { PublicCommand::Unknown => {}, other => panic!("Expected Unknown for suffix variant, got {:?}", other) }
}

#[test]
fn test_alternate_prefixes_exclamation_and_slash() {
    let parser = PublicCommandParser::new_with_prefixes(Some("!/^".to_string()));
    match parser.parse("!HELP") { PublicCommand::Help => {}, other => panic!("Expected Help with '!' prefix, got {:?}", other) }
    match parser.parse("/LOGIN Bob") { PublicCommand::Login(u) => assert_eq!(u, "Bob"), other => panic!("Expected Login with '/' prefix, got {:?}", other) }
    // Non-allowed prefix should not parse
    match parser.parse("#SLOT") { PublicCommand::Unknown => {}, other => panic!("Expected Unknown for '#' prefix, got {:?}", other) }
}