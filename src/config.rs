use serde::Deserialize;

const DEFAULT_FORMAT: &str = "{index}. {title}";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    DEFAULT_FORMAT.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: default_format(),
        }
    }
}
