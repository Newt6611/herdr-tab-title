use super::RenderContext;
use super::token::{Placeholder, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    tokens: Vec<Token>,
}

impl Template {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let mut tokens = Vec::new();
        let mut literal = String::new();
        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            if character != '{' {
                literal.push(character);
                continue;
            }

            if !literal.is_empty() {
                tokens.push(Token::Literal(std::mem::take(&mut literal)));
            }

            let mut name = String::new();
            let mut closed = false;

            for placeholder_char in chars.by_ref() {
                if placeholder_char == '}' {
                    closed = true;
                    break;
                }

                name.push(placeholder_char);
            }

            if !closed {
                return Err(ParseError::UnclosedPlaceholder);
            }

            if name.is_empty() {
                return Err(ParseError::EmptyPlaceholder);
            }

            let Some(placeholder) = Placeholder::parse(&name) else {
                return Err(ParseError::UnknownPlaceholder(name));
            };

            tokens.push(Token::Placeholder(placeholder));
        }

        if !literal.is_empty() {
            tokens.push(Token::Literal(literal));
        }

        Ok(Self { tokens })
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn render(&self, context: &RenderContext<'_>) -> String {
        let mut output = String::new();

        for token in &self.tokens {
            match token {
                Token::Literal(value) => output.push_str(value),
                Token::Placeholder(Placeholder::Index) => {
                    output.push_str(&context.index.to_string());
                }
                Token::Placeholder(Placeholder::Title) => output.push_str(context.title),
            }
        }

        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnknownPlaceholder(String),
    UnclosedPlaceholder,
    EmptyPlaceholder,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPlaceholder(name) => write!(f, "unknown placeholder {{{name}}}"),
            Self::UnclosedPlaceholder => write!(f, "unclosed placeholder"),
            Self::EmptyPlaceholder => write!(f, "empty placeholder"),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_literal_only_template() {
        let template = Template::parse("tabs").unwrap();

        assert_eq!(template.tokens(), &[Token::Literal("tabs".to_string())]);
    }

    #[test]
    fn parses_adjacent_placeholders() {
        let template = Template::parse("{index}{title}").unwrap();

        assert_eq!(
            template.tokens(),
            &[
                Token::Placeholder(Placeholder::Index),
                Token::Placeholder(Placeholder::Title),
            ]
        );
    }

    #[test]
    fn parses_mixed_literals_and_placeholders() {
        let template = Template::parse("[{index}] {title}").unwrap();

        assert_eq!(
            template.tokens(),
            &[
                Token::Literal("[".to_string()),
                Token::Placeholder(Placeholder::Index),
                Token::Literal("] ".to_string()),
                Token::Placeholder(Placeholder::Title),
            ]
        );
    }

    #[test]
    fn rejects_unknown_placeholders() {
        let error = Template::parse("{workspace}").unwrap_err();

        assert_eq!(
            error,
            ParseError::UnknownPlaceholder("workspace".to_string())
        );
    }

    #[test]
    fn rejects_empty_placeholders() {
        let error = Template::parse("{}").unwrap_err();

        assert_eq!(error, ParseError::EmptyPlaceholder);
    }

    #[test]
    fn rejects_unclosed_placeholders() {
        let error = Template::parse("{index").unwrap_err();

        assert_eq!(error, ParseError::UnclosedPlaceholder);
    }

    #[test]
    fn keeps_closing_braces_in_literals() {
        let template = Template::parse("{title}}").unwrap();

        assert_eq!(
            template.tokens(),
            &[
                Token::Placeholder(Placeholder::Title),
                Token::Literal("}".to_string()),
            ]
        );
    }
}
