use std::collections::HashMap;
use std::path::Path;

use crate::formatter::{Formatter, RenderContext};
use herdr_plugin::{Pane, Tab};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenameOperation {
    tab_id: String,
    from: String,
    to: String,
}

fn plan_renames(tabs: &[Tab], formatter: &Formatter) -> Vec<RenameOperation> {
    plan_renames_with_title_overrides(tabs, formatter, &HashMap::new())
}

fn plan_renames_with_title_overrides(
    tabs: &[Tab],
    formatter: &Formatter,
    title_overrides: &HashMap<String, String>,
) -> Vec<RenameOperation> {
    let mut operations = Vec::new();
    let mut tabs = tabs.iter().collect::<Vec<_>>();
    tabs.sort_by(|left, right| {
        left.workspace_id
            .cmp(&right.workspace_id)
            .then(left.number.cmp(&right.number))
    });

    let mut current_workspace = None::<&str>;
    let mut workspace_index = 0usize;

    for tab in tabs {
        if current_workspace != Some(tab.workspace_id.as_str()) {
            current_workspace = Some(tab.workspace_id.as_str());
            workspace_index = 1;
        } else {
            workspace_index += 1;
        }

        let clean_title = title_overrides
            .get(&tab.tab_id)
            .map(String::as_str)
            .unwrap_or_else(|| formatter.clean_title(&tab.label));
        let expected = formatter.render(&RenderContext {
            index: workspace_index,
            title: clean_title,
        });

        if tab.label != expected {
            operations.push(RenameOperation {
                tab_id: tab.tab_id.clone(),
                from: tab.label.clone(),
                to: expected,
            });
        }
    }

    operations
}

pub async fn refresh_sdk(
    client: &herdr_plugin::HerdrClient,
    formatter: &Formatter,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let tabs = list_tabs(client).await?;
    let operations = plan_renames(&tabs, formatter);
    apply_sdk_renames(client, operations).await
}

pub async fn refresh_created_tab_sdk(
    client: &herdr_plugin::HerdrClient,
    formatter: &Formatter,
    tab_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let tabs = list_tabs(client).await?;
    let panes = list_panes(client).await?;
    let title_overrides = created_tab_title(&tabs, &panes, formatter, tab_id)
        .map(|title| HashMap::from([(tab_id.to_string(), title)]))
        .unwrap_or_default();
    let operations = plan_renames_with_title_overrides(&tabs, formatter, &title_overrides);
    apply_sdk_renames(client, operations).await
}

async fn apply_sdk_renames(
    client: &herdr_plugin::HerdrClient,
    operations: Vec<RenameOperation>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let count = operations.len();

    for operation in operations {
        client
            .tab()
            .rename(&operation.tab_id, &operation.to)
            .await?;
    }

    Ok(count)
}

async fn list_tabs(
    client: &herdr_plugin::HerdrClient,
) -> Result<Vec<Tab>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(client
        .tab()
        .list(herdr_plugin::TabListOptions::default())
        .await?
        .tabs)
}

async fn list_panes(
    client: &herdr_plugin::HerdrClient,
) -> Result<Vec<Pane>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(client
        .pane()
        .list(herdr_plugin::PaneListOptions::default())
        .await?
        .panes)
}

fn created_tab_title(
    tabs: &[Tab],
    panes: &[Pane],
    formatter: &Formatter,
    tab_id: &str,
) -> Option<String> {
    let tab = tabs.iter().find(|tab| tab.tab_id == tab_id)?;
    if !formatter.clean_title(&tab.label).trim().is_empty() {
        return None;
    }

    panes
        .iter()
        .find(|pane| pane.tab_id == tab_id)
        .and_then(|pane| {
            pane.foreground_cwd
                .to_str()
                .and_then(path_basename)
                .or_else(|| pane.cwd.to_str().and_then(path_basename))
                .map(str::to_string)
        })
}

fn path_basename(path: &str) -> Option<&str> {
    Path::new(path)
        .file_name()?
        .to_str()
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn tab(id: &str, title: &str, workspace_id: &str, number: u64) -> Tab {
        Tab {
            workspace_id: workspace_id.to_string(),
            tab_id: id.to_string(),
            agent_status: "unknown".to_string(),
            focused: false,
            label: title.to_string(),
            number,
            pane_count: 1,
        }
    }

    fn pane(tab_id: &str, cwd: &str, foreground_cwd: &str) -> Pane {
        Pane {
            workspace_id: "w1".to_string(),
            tab_id: tab_id.to_string(),
            pane_id: format!("{tab_id}:p1"),
            terminal_id: format!("{tab_id}:terminal"),
            agent_status: "unknown".to_string(),
            cwd: PathBuf::from(cwd),
            foreground_cwd: PathBuf::from(foreground_cwd),
            focused: false,
            revision: 1,
            agent: None,
            agent_session: None,
            label: None,
        }
    }

    #[test]
    fn plans_only_titles_that_need_changes() {
        let tabs = vec![
            tab("t1", "1. Codex", "w1", 1),
            tab("t2", "Terminal", "w1", 2),
            tab("t3", "9. Claude 3.5", "w1", 3),
        ];
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(
            operations,
            vec![
                RenameOperation {
                    tab_id: "t2".to_string(),
                    from: "Terminal".to_string(),
                    to: "2. Terminal".to_string(),
                },
                RenameOperation {
                    tab_id: "t3".to_string(),
                    from: "9. Claude 3.5".to_string(),
                    to: "3. Claude 3.5".to_string(),
                }
            ]
        );
    }

    #[test]
    fn plans_only_changed_tabs_without_refocusing() {
        let tabs = vec![tab("t1", "Codex", "w1", 1), tab("t2", "2. Claude", "w1", 2)];
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(
            operations,
            vec![RenameOperation {
                tab_id: "t1".to_string(),
                from: "Codex".to_string(),
                to: "1. Codex".to_string(),
            }]
        );
    }

    #[test]
    fn unmatched_trimmed_label_is_used_as_title() {
        let tabs = vec![tab("t1", "1.", "w1", 1)];
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(
            operations,
            vec![RenameOperation {
                tab_id: "t1".to_string(),
                from: "1.".to_string(),
                to: "1. 1.".to_string(),
            }]
        );
    }

    #[test]
    fn custom_index_format_does_not_nest_existing_prefix() {
        let tabs = vec![
            tab("t1", "[1] Codex", "w1", 1),
            tab("t2", "[2] Claude", "w1", 2),
        ];
        let formatter = Formatter::parse("[{index}] {title}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(operations, Vec::<RenameOperation>::new());
    }

    #[test]
    fn repeated_index_format_extracts_title_when_tab_moves() {
        let tabs = vec![
            tab("t1", "[2] 2 Claude", "w1", 1),
            tab("t2", "[1] 1 Codex", "w1", 2),
        ];
        let formatter = Formatter::parse("[{index}] {index} {title}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(
            operations,
            vec![
                RenameOperation {
                    tab_id: "t1".to_string(),
                    from: "[2] 2 Claude".to_string(),
                    to: "[1] 1 Claude".to_string(),
                },
                RenameOperation {
                    tab_id: "t2".to_string(),
                    from: "[1] 1 Codex".to_string(),
                    to: "[2] 2 Codex".to_string(),
                },
            ]
        );
    }

    #[test]
    fn titleless_repeated_index_format_settles_after_move() {
        let tabs = vec![tab("t1", "[2] 2", "w1", 1), tab("t2", "[1] 1", "w1", 2)];
        let formatter = Formatter::parse("[{index}] {index}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(
            operations,
            vec![
                RenameOperation {
                    tab_id: "t1".to_string(),
                    from: "[2] 2".to_string(),
                    to: "[1] 1".to_string(),
                },
                RenameOperation {
                    tab_id: "t2".to_string(),
                    from: "[1] 1".to_string(),
                    to: "[2] 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn refresh_created_tab_uses_pane_cwd_basename_as_title() {
        let tabs = vec![
            tab("t1", "1. Codex", "w1", 1),
            tab("t2", "2. Claude", "w1", 2),
            tab("t3", "", "w1", 3),
        ];
        let panes = vec![pane("t3", "/Users/newt/dev/herdr", "")];
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let title_overrides = created_tab_title(&tabs, &panes, &formatter, "t3")
            .map(|title| HashMap::from([("t3".to_string(), title)]))
            .unwrap_or_default();
        let operations = plan_renames_with_title_overrides(&tabs, &formatter, &title_overrides);

        assert_eq!(
            operations,
            vec![RenameOperation {
                tab_id: "t3".to_string(),
                from: "".to_string(),
                to: "3. herdr".to_string(),
            }]
        );
    }

    #[test]
    fn refresh_created_tab_detects_empty_title_from_current_format() {
        let tabs = vec![
            tab("t1", "tab 1: Codex!", "w1", 1),
            tab("t2", "tab 2: !", "w1", 2),
        ];
        let panes = vec![pane("t2", "/Users/newt/dev/herdr-tab-title", "")];
        let formatter = Formatter::parse("tab {index}: {title}!").unwrap();

        let title_overrides = created_tab_title(&tabs, &panes, &formatter, "t2")
            .map(|title| HashMap::from([("t2".to_string(), title)]))
            .unwrap_or_default();
        let operations = plan_renames_with_title_overrides(&tabs, &formatter, &title_overrides);

        assert_eq!(
            operations,
            vec![RenameOperation {
                tab_id: "t2".to_string(),
                from: "tab 2: !".to_string(),
                to: "tab 2: herdr-tab-title!".to_string(),
            }]
        );
    }

    #[test]
    fn refresh_created_tab_preserves_custom_title() {
        let tabs = vec![tab("t1", "server logs", "w1", 1)];
        let panes = vec![pane("t1", "/Users/newt/dev/herdr", "")];
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let title_overrides = created_tab_title(&tabs, &panes, &formatter, "t1")
            .map(|title| HashMap::from([("t1".to_string(), title)]))
            .unwrap_or_default();
        let operations = plan_renames_with_title_overrides(&tabs, &formatter, &title_overrides);

        assert_eq!(
            operations,
            vec![RenameOperation {
                tab_id: "t1".to_string(),
                from: "server logs".to_string(),
                to: "1. server logs".to_string(),
            }]
        );
    }

    #[test]
    fn plans_indexes_per_workspace_order() {
        let tabs = vec![
            tab("w2t2", "two", "w2", 2),
            tab("w1t2", "two", "w1", 2),
            tab("w2t1", "one", "w2", 1),
            tab("w1t1", "one", "w1", 1),
        ];
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let operations = plan_renames(&tabs, &formatter);

        assert_eq!(
            operations,
            vec![
                RenameOperation {
                    tab_id: "w1t1".to_string(),
                    from: "one".to_string(),
                    to: "1. one".to_string(),
                },
                RenameOperation {
                    tab_id: "w1t2".to_string(),
                    from: "two".to_string(),
                    to: "2. two".to_string(),
                },
                RenameOperation {
                    tab_id: "w2t1".to_string(),
                    from: "one".to_string(),
                    to: "1. one".to_string(),
                },
                RenameOperation {
                    tab_id: "w2t2".to_string(),
                    from: "two".to_string(),
                    to: "2. two".to_string(),
                },
            ]
        );
    }
}
