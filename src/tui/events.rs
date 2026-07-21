use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};

pub fn next(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
