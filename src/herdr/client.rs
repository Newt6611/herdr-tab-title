use std::path::PathBuf;
use std::process::Command;

use super::commands;
use super::models::{Pane, Tab, Workspace};

pub trait HerdrApi {
    fn list_tabs(&self) -> Result<Vec<Tab>, HerdrError>;
    fn rename_tab(&self, id: &str, title: &str) -> Result<(), HerdrError>;
    fn focus_tab(&self, id: &str) -> Result<(), HerdrError>;
    fn list_panes(&self) -> Result<Vec<Pane>, HerdrError>;
    fn list_workspaces(&self) -> Result<Vec<Workspace>, HerdrError>;
}

#[derive(Debug, Clone)]
pub struct HerdrClient {
    bin: PathBuf,
}

impl HerdrClient {
    pub fn from_env() -> Self {
        Self {
            bin: std::env::var_os("HERDR_BIN_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("herdr")),
        }
    }

    pub fn new(bin: PathBuf) -> Self {
        Self { bin }
    }
}

impl HerdrApi for HerdrClient {
    fn list_tabs(&self) -> Result<Vec<Tab>, HerdrError> {
        let response: CliResponse<TabListResult> = self.run_json(&commands::list_tabs_args())?;
        Ok(response.result.tabs)
    }

    fn rename_tab(&self, id: &str, title: &str) -> Result<(), HerdrError> {
        self.run_unit(&commands::rename_tab_args(id, title))
    }

    fn focus_tab(&self, id: &str) -> Result<(), HerdrError> {
        self.run_unit(&commands::focus_tab_args(id))
    }

    fn list_panes(&self) -> Result<Vec<Pane>, HerdrError> {
        let response: CliResponse<PaneListResult> = self.run_json(&commands::list_panes_args())?;
        Ok(response.result.panes)
    }

    fn list_workspaces(&self) -> Result<Vec<Workspace>, HerdrError> {
        let response: CliResponse<WorkspaceListResult> =
            self.run_json(&commands::list_workspaces_args())?;
        Ok(response.result.workspaces)
    }
}

impl HerdrClient {
    fn run_json<T>(&self, args: &[String]) -> Result<T, HerdrError>
    where
        T: serde::de::DeserializeOwned,
    {
        let output = self.run(args)?;
        serde_json::from_slice(&output.stdout).map_err(HerdrError::Json)
    }

    fn run_unit(&self, args: &[String]) -> Result<(), HerdrError> {
        self.run(args).map(|_| ())
    }

    fn run(&self, args: &[String]) -> Result<std::process::Output, HerdrError> {
        let output = Command::new(&self.bin)
            .args(args)
            .output()
            .map_err(HerdrError::Spawn)?;

        if output.status.success() {
            return Ok(output);
        }

        Err(HerdrError::Failed {
            command: format!("{} {}", self.bin.display(), args.join(" ")),
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

#[derive(Debug)]
pub enum HerdrError {
    Spawn(std::io::Error),
    Failed {
        command: String,
        status: Option<i32>,
        stderr: String,
    },
    Json(serde_json::Error),
}

impl std::fmt::Display for HerdrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "failed to run herdr: {error}"),
            Self::Failed {
                command,
                status,
                stderr,
            } => write!(
                f,
                "herdr command failed ({command}, status {status:?}): {stderr}"
            ),
            Self::Json(error) => write!(f, "failed to parse herdr JSON: {error}"),
        }
    }
}

impl std::error::Error for HerdrError {}

#[derive(Debug, serde::Deserialize)]
struct CliResponse<T> {
    result: T,
}

#[derive(Debug, serde::Deserialize)]
struct TabListResult {
    tabs: Vec<Tab>,
}

#[derive(Debug, serde::Deserialize)]
struct PaneListResult {
    panes: Vec<Pane>,
}

#[derive(Debug, serde::Deserialize)]
struct WorkspaceListResult {
    workspaces: Vec<Workspace>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tab_list_response_from_cli_json() {
        let response: CliResponse<TabListResult> = serde_json::from_str(
            r#"{
                "id": "cli:tab:list",
                "result": {
                    "type": "tab_list",
                    "tabs": [
                        {
                            "tab_id": "w1:t2",
                            "workspace_id": "w1",
                            "label": "codex",
                            "number": 2,
                            "focused": true,
                            "pane_count": 1,
                            "agent_status": "idle"
                        }
                    ]
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            response.result.tabs,
            vec![Tab {
                id: "w1:t2".to_string(),
                title: "codex".to_string(),
                workspace_id: "w1".to_string(),
                number: 2,
                focused: true,
            }]
        );
    }
}
