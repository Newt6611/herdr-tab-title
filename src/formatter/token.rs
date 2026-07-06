#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Literal(String),
    Placeholder(Placeholder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placeholder {
    Index,
    Title,
}

impl Placeholder {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "index" => Some(Self::Index),
            "title" => Some(Self::Title),
            _ => None,
        }
    }
}
