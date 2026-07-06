pub fn list_tabs_args() -> Vec<String> {
    ["tab", "list"].into_iter().map(String::from).collect()
}

pub fn rename_tab_args(id: &str, title: &str) -> Vec<String> {
    vec![
        "tab".to_string(),
        "rename".to_string(),
        id.to_string(),
        title.to_string(),
    ]
}

pub fn focus_tab_args(id: &str) -> Vec<String> {
    vec!["tab".to_string(), "focus".to_string(), id.to_string()]
}

pub fn list_workspaces_args() -> Vec<String> {
    ["workspace", "list"]
        .into_iter()
        .map(String::from)
        .collect()
}
