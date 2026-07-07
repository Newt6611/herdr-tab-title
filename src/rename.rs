use crate::formatter::{Formatter, RenderContext, strip_numeric_prefix};
use crate::herdr::client::{HerdrApi, HerdrError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameOperation {
    pub tab_id: String,
    pub from: String,
    pub to: String,
}

pub fn plan_renames<C: HerdrApi>(
    client: &C,
    formatter: &Formatter,
) -> Result<Vec<RenameOperation>, HerdrError> {
    let tabs = client.list_tabs()?;

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

        let clean_title = strip_numeric_prefix(&tab.title);
        let expected = formatter.render(&RenderContext {
            index: workspace_index,
            title: clean_title,
        });

        if tab.title != expected {
            operations.push(RenameOperation {
                tab_id: tab.id.clone(),
                from: tab.title.clone(),
                to: expected,
            });
        }
    }

    Ok(operations)
}

pub fn refresh<C: HerdrApi>(client: &C, formatter: &Formatter) -> Result<usize, HerdrError> {
    let operations = plan_renames(client, formatter)?;
    let count = operations.len();

    for operation in operations {
        client.rename_tab(&operation.tab_id, &operation.to)?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::herdr::models::{Tab, Workspace};

    use super::*;

    #[derive(Debug)]
    struct MockHerdrClient {
        tabs: Vec<Tab>,
        renames: RefCell<Vec<(String, String)>>,
    }

    impl HerdrApi for MockHerdrClient {
        fn list_tabs(&self) -> Result<Vec<Tab>, HerdrError> {
            Ok(self.tabs.clone())
        }

        fn rename_tab(&self, id: &str, title: &str) -> Result<(), HerdrError> {
            self.renames
                .borrow_mut()
                .push((id.to_string(), title.to_string()));
            Ok(())
        }

        fn focus_tab(&self, id: &str) -> Result<(), HerdrError> {
            self.renames
                .borrow_mut()
                .push((id.to_string(), "<focus>".to_string()));
            Ok(())
        }

        fn list_workspaces(&self) -> Result<Vec<Workspace>, HerdrError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn plans_only_titles_that_need_changes() {
        let client = MockHerdrClient {
            tabs: vec![
                Tab {
                    id: "t1".to_string(),
                    title: "1. Codex".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 1,
                    focused: false,
                },
                Tab {
                    id: "t2".to_string(),
                    title: "Terminal".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 2,
                    focused: false,
                },
                Tab {
                    id: "t3".to_string(),
                    title: "9. Claude 3.5".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 3,
                    focused: false,
                },
            ],
            renames: RefCell::new(Vec::new()),
        };
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let operations = plan_renames(&client, &formatter).unwrap();

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
    fn refresh_renames_only_changed_tabs_without_refocusing() {
        let client = MockHerdrClient {
            tabs: vec![
                Tab {
                    id: "t1".to_string(),
                    title: "Codex".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 1,
                    focused: true,
                },
                Tab {
                    id: "t2".to_string(),
                    title: "2. Claude".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 2,
                    focused: false,
                },
            ],
            renames: RefCell::new(Vec::new()),
        };
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let count = refresh(&client, &formatter).unwrap();

        assert_eq!(count, 1);
        assert_eq!(
            client.renames.borrow().as_slice(),
            &[("t1".to_string(), "1. Codex".to_string())]
        );
    }

    #[test]
    fn plans_indexes_per_workspace_order() {
        let client = MockHerdrClient {
            tabs: vec![
                Tab {
                    id: "w2t2".to_string(),
                    title: "two".to_string(),
                    workspace_id: "w2".to_string(),
                    number: 2,
                    focused: false,
                },
                Tab {
                    id: "w1t2".to_string(),
                    title: "two".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 2,
                    focused: false,
                },
                Tab {
                    id: "w2t1".to_string(),
                    title: "one".to_string(),
                    workspace_id: "w2".to_string(),
                    number: 1,
                    focused: false,
                },
                Tab {
                    id: "w1t1".to_string(),
                    title: "one".to_string(),
                    workspace_id: "w1".to_string(),
                    number: 1,
                    focused: false,
                },
            ],
            renames: RefCell::new(Vec::new()),
        };
        let formatter = Formatter::parse("{index}. {title}").unwrap();

        let operations = plan_renames(&client, &formatter).unwrap();

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
