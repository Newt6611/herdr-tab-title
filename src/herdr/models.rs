use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tab {
    #[serde(alias = "tab_id")]
    pub id: String,
    #[serde(alias = "label")]
    pub title: String,
    pub workspace_id: String,
    pub number: usize,
    #[serde(default)]
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Workspace {
    #[serde(alias = "workspace_id")]
    pub id: String,
    #[serde(alias = "label")]
    pub title: String,
}
