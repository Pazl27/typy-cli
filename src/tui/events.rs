use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};

/// Poll for a single terminal event, waiting at most `timeout`.
///
/// Returns `Ok(None)` when the timeout elapses with no event, which lets the
/// main loop keep redrawing (e.g. for the live timer) even when the user isn't
/// pressing anything.
pub fn next(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
