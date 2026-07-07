use std::path::PathBuf;

use herdr_tab_title::config::Config;
use herdr_tab_title::debounce::{DebounceError, run_cross_process};
use herdr_tab_title::events::{REFRESH_DEBOUNCE, is_refresh_event};
use herdr_tab_title::formatter::Formatter;
use herdr_tab_title::herdr::client::HerdrClient;
use herdr_tab_title::rename;

fn main() {
    if let Err(error) = run() {
        eprintln!("herdr-tab-title: {error}");
    }
}

fn run() -> Result<(), String> {
    let config = Config::load().map_err(|error| error.to_string())?;
    let formatter = Formatter::parse(&config.format).map_err(|error| error.to_string())?;
    let client = HerdrClient::from_env();

    let event_name = std::env::var("HERDR_PLUGIN_EVENT").ok();
    if let Some(event_name) = event_name.as_deref() {
        if !is_refresh_event(event_name) {
            return Ok(());
        }
    }

    let state_dir = std::env::var_os("HERDR_PLUGIN_STATE_DIR").map(PathBuf::from);
    run_cross_process(state_dir.as_deref(), REFRESH_DEBOUNCE, || {
        if event_name.as_deref() == Some("tab.created") {
            if let Ok(tab_id) = std::env::var("HERDR_TAB_ID") {
                rename::refresh_created_tab(&client, &formatter, &tab_id)
                    .map_err(|error| DebounceError::Action(error.to_string()))?;
                return Ok(());
            }
        }

        rename::refresh(&client, &formatter)
            .map_err(|error| DebounceError::Action(error.to_string()))?;

        Ok(())
    })
    .map(|_| ())
    .map_err(|error| error.to_string())
}
