use std::time::Duration;

pub const REFRESH_DEBOUNCE: Duration = Duration::from_millis(100);

pub const RELEVANT_EVENTS: &[&str] = &[
    "tab.created",
    "tab.renamed",
    "tab.closed",
    "tab.focused",
    "workspace.created",
    "workspace.focused",
    "workspace.renamed",
    "workspace.closed",
];

pub fn is_refresh_event(event: &str) -> bool {
    RELEVANT_EVENTS.contains(&event)
}

pub fn subscriptions_json() -> serde_json::Value {
    serde_json::json!({
        "subscriptions": RELEVANT_EVENTS
            .iter()
            .map(|event| serde_json::json!({ "type": event }))
            .collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_unknown_tab_reorder_events() {
        assert!(!is_refresh_event("tab.moved"));
        assert!(!is_refresh_event("tab.reordered"));
    }
}
