pub mod parser;
pub mod token;

use parser::Template;
use regex::Regex;
use token::{Placeholder, Token};

pub use parser::ParseError;

#[derive(Debug, Clone)]
pub struct Formatter {
    template: Template,
    matcher: FormatMatcher,
}

#[derive(Debug, Clone)]
struct FormatMatcher {
    regex: Regex,
    captures_title: bool,
    trailing_empty_title_padding: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderContext<'a> {
    pub index: usize,
    pub title: &'a str,
}

impl Formatter {
    pub fn parse(format: &str) -> Result<Self, ParseError> {
        let template = Template::parse(format)?;
        let matcher = build_matcher(template.tokens());

        Ok(Self { template, matcher })
    }

    pub fn render(&self, context: &RenderContext<'_>) -> String {
        self.template.render(context)
    }

    pub fn clean_title<'a>(&self, title: &'a str) -> &'a str {
        if let Some(clean_title) = self.capture_title(title) {
            return clean_title;
        }

        if let Some(padding) = &self.matcher.trailing_empty_title_padding {
            let padded_title = format!("{title}{padding}");
            if self.capture_title(&padded_title) == Some("") {
                return "";
            }
        }

        title
    }

    fn capture_title<'a>(&self, title: &'a str) -> Option<&'a str> {
        let captures = self.matcher.regex.captures(title)?;

        if self.matcher.captures_title {
            captures.name("title").map(|title| title.as_str())
        } else {
            Some("")
        }
    }
}

fn build_matcher(tokens: &[Token]) -> FormatMatcher {
    let title_count = tokens
        .iter()
        .filter(|token| matches!(token, Token::Placeholder(Placeholder::Title)))
        .count();
    let captures_title = title_count == 1;

    let mut pattern = String::from("^");

    for token in tokens {
        match token {
            Token::Literal(value) => pattern.push_str(&regex::escape(value)),
            Token::Placeholder(Placeholder::Index) => pattern.push_str(r"\d+"),
            Token::Placeholder(Placeholder::Title) if captures_title => {
                pattern.push_str(r"(?P<title>.*?)");
            }
            Token::Placeholder(Placeholder::Title) => pattern.push_str(r".*?"),
        }
    }

    pattern.push('$');

    FormatMatcher {
        regex: Regex::new(&pattern).expect("generated formatter regex should be valid"),
        captures_title,
        trailing_empty_title_padding: trailing_empty_title_padding(tokens, captures_title),
    }
}

fn trailing_empty_title_padding(tokens: &[Token], captures_title: bool) -> Option<String> {
    if !captures_title || !matches!(tokens.last(), Some(Token::Placeholder(Placeholder::Title))) {
        return None;
    }

    let Some(Token::Literal(literal)) = tokens.iter().rev().nth(1) else {
        return None;
    };

    let padding = literal
        .chars()
        .rev()
        .take_while(|character| character.is_whitespace())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();

    (!padding.is_empty()).then_some(padding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::token::{Placeholder, Token};

    #[test]
    fn renders_index_and_title_placeholders() {
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        assert_eq!(
            formatter.render(&RenderContext {
                index: 3,
                title: "Claude"
            }),
            "3. Claude"
        );
    }

    #[test]
    fn cleans_title_wrapped_by_current_format() {
        let formatter = Formatter::parse("[{index}] {title}").unwrap();

        assert_eq!(formatter.clean_title("[2] Claude"), "Claude");
    }

    #[test]
    fn cleans_title_wrapped_by_format_with_suffix() {
        let formatter = Formatter::parse("tab {index}: {title}!").unwrap();

        assert_eq!(formatter.clean_title("tab 12: Claude!"), "Claude");
    }

    #[test]
    fn cleans_title_when_index_is_after_title() {
        let formatter = Formatter::parse("{title} [{index}]").unwrap();

        assert_eq!(formatter.clean_title("Claude [12]"), "Claude");
    }

    #[test]
    fn cleans_title_with_regex_metacharacter_literals() {
        let formatter = Formatter::parse("[{index}] ({title}) + ${index}?").unwrap();

        assert_eq!(formatter.clean_title("[12] (Claude) + $12?"), "Claude");
    }

    #[test]
    fn titleless_format_does_not_extract_title() {
        let formatter = Formatter::parse("[{index}] {index}").unwrap();

        assert_eq!(formatter.clean_title("[12] 12"), "");
    }

    #[test]
    fn clean_title_extracts_explicit_empty_trailing_title() {
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        assert_eq!(formatter.clean_title("1. "), "");
    }

    #[test]
    fn clean_title_accepts_trimmed_empty_trailing_title() {
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        assert_eq!(formatter.clean_title("1."), "");
    }

    #[test]
    fn matcher_treats_index_as_digits() {
        let template = Template::parse("[{index}] {title}").unwrap();
        let matcher = build_matcher(template.tokens());
        let captures = matcher.regex.captures("[987] Claude").unwrap();

        assert!(matcher.captures_title);
        assert_eq!(captures.name("title").unwrap().as_str(), "Claude");
    }

    #[test]
    fn matcher_anchors_and_escapes_literals() {
        let template = Template::parse("({index}) {title}?").unwrap();
        let matcher = build_matcher(template.tokens());

        assert!(matcher.regex.is_match("(3) Claude?"));
        assert!(!matcher.regex.is_match("x(3) Claude?"));
        assert!(!matcher.regex.is_match("(3) Claude?!"));
        assert!(!matcher.regex.is_match("3 Claude?"));
    }

    #[test]
    fn matcher_without_exactly_one_title_does_not_capture_title() {
        let titleless = Template::parse("[{index}] {index}").unwrap();
        let titleless_matcher = build_matcher(titleless.tokens());
        assert!(!titleless_matcher.captures_title);
        assert!(titleless_matcher.regex.is_match("[12] 12"));

        let duplicate_title = Template::parse("{title} / {title}").unwrap();
        let duplicate_matcher = build_matcher(duplicate_title.tokens());
        assert!(!duplicate_matcher.captures_title);
        assert!(duplicate_matcher.regex.is_match("left / right"));
    }

    #[test]
    fn parses_template_into_tokens() {
        let template = Template::parse("{index}. {title}").unwrap();

        assert_eq!(
            template.tokens(),
            &[
                Token::Placeholder(Placeholder::Index),
                Token::Literal(". ".to_string()),
                Token::Placeholder(Placeholder::Title)
            ]
        );
    }
}
