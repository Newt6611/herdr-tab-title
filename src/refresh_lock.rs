use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const REFRESH_LOCK_DELAY: Duration = Duration::from_millis(100);

pub fn run_cross_process<F>(
    state_dir: Option<&Path>,
    delay: Duration,
    mut action: F,
) -> Result<bool, RefreshLockError>
where
    F: FnMut() -> Result<(), RefreshLockError>,
{
    let Some(state_dir) = state_dir else {
        std::thread::sleep(delay);
        action()?;
        return Ok(true);
    };

    std::fs::create_dir_all(state_dir).map_err(RefreshLockError::Io)?;
    let marker_path = state_dir.join("tab-title-refresh-lock-ms");
    let deadline = unix_ms()? + delay.as_millis() as u128;
    let deadline_marker = deadline.to_string();
    std::fs::write(&marker_path, &deadline_marker).map_err(RefreshLockError::Io)?;

    std::thread::sleep(delay);

    let latest = std::fs::read_to_string(&marker_path).map_err(RefreshLockError::Io)?;
    if latest.trim() == deadline_marker {
        action()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn unix_ms() -> Result<u128, RefreshLockError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(RefreshLockError::Clock)?
        .as_millis())
}

#[derive(Debug)]
pub enum RefreshLockError {
    Io(std::io::Error),
    Clock(std::time::SystemTimeError),
    Action(String),
}

impl std::fmt::Display for RefreshLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "refresh lock I/O failed: {error}"),
            Self::Clock(error) => write!(f, "refresh lock clock failed: {error}"),
            Self::Action(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RefreshLockError {}
