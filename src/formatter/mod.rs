pub mod parser;
pub mod token;

use parser::Template;

pub use parser::ParseError;

#[derive(Debug, Clone)]
pub struct Formatter {
    template: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderContext<'a> {
    pub index: usize,
    pub title: &'a str,
}

impl Formatter {
    pub fn parse(format: &str) -> Result<Self, ParseError> {
        Ok(Self {
            template: Template::parse(format)?,
        })
    }

    pub fn render(&self, context: &RenderContext<'_>) -> String {
        self.template.render(context)
    }
}

pub fn strip_numeric_prefix(title: &str) -> &str {
    let Some(dot_index) = title.find(". ") else {
        return title;
    };

    if dot_index == 0 {
        return title;
    }

    if title[..dot_index]
        .chars()
        .all(|character| character.is_ascii_digit())
    {
        &title[dot_index + 2..]
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::token::{Placeholder, Token};

    #[test]
    fn strips_only_leading_numeric_prefixes() {
        assert_eq!(strip_numeric_prefix("1. Claude"), "Claude");
        assert_eq!(strip_numeric_prefix("12. Backend"), "Backend");
        assert_eq!(strip_numeric_prefix("Claude 3.5"), "Claude 3.5");
        assert_eq!(
            strip_numeric_prefix("Claude 3. Backend"),
            "Claude 3. Backend"
        );
        assert_eq!(strip_numeric_prefix(". Claude"), ". Claude");
    }

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
