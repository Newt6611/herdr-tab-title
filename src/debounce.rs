use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Debouncer {
    delay: Duration,
    deadline: Option<Instant>,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            deadline: None,
        }
    }

    pub fn trigger(&mut self) {
        self.deadline = Some(Instant::now() + self.delay);
    }

    pub fn is_ready(&self) -> bool {
        self.deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    }

    pub fn clear(&mut self) {
        self.deadline = None;
    }
}

pub fn run_cross_process<F>(
    state_dir: Option<&Path>,
    delay: Duration,
    mut action: F,
) -> Result<bool, DebounceError>
where
    F: FnMut() -> Result<(), DebounceError>,
{
    let Some(state_dir) = state_dir else {
        std::thread::sleep(delay);
        action()?;
        return Ok(true);
    };

    std::fs::create_dir_all(state_dir).map_err(DebounceError::Io)?;
    let marker_path = state_dir.join("tab-title-debounce-ms");
    let deadline = unix_ms()? + delay.as_millis() as u128;
    let deadline_marker = deadline.to_string();
    std::fs::write(&marker_path, &deadline_marker).map_err(DebounceError::Io)?;

    std::thread::sleep(delay);

    let latest = std::fs::read_to_string(&marker_path).map_err(DebounceError::Io)?;
    if latest.trim() == deadline_marker {
        action()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn unix_ms() -> Result<u128, DebounceError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(DebounceError::Clock)?
        .as_millis())
}

#[derive(Debug)]
pub enum DebounceError {
    Io(std::io::Error),
    Clock(std::time::SystemTimeError),
    Action(String),
}

impl std::fmt::Display for DebounceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "debounce I/O failed: {error}"),
            Self::Clock(error) => write!(f, "debounce clock failed: {error}"),
            Self::Action(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DebounceError {}
