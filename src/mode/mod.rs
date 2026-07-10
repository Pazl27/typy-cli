mod mode_selector;

pub use mode_selector::{Mode, ModeType};

use std::str::FromStr;

/// Parse a comma-separated list of mode names into `ModeType`s (unknown names
/// are dropped). If `normal` appears, the result collapses to just `Normal`.
pub fn parse_modes(s: &str) -> Vec<ModeType> {
    let modes: Vec<ModeType> = s
        .split(',')
        .filter_map(|m| ModeType::from_str(m.trim()).ok())
        .collect();
    if modes.contains(&ModeType::Normal) || modes.is_empty() {
        vec![ModeType::Normal]
    } else {
        modes
    }
}
