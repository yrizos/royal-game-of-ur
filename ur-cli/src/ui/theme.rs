use ratatui::style::Color;
use std::time::Duration;

/// Player 1 (human) colour — light blue throughout the UI.
pub const COLOR_P1: Color = Color::LightBlue;
/// Player 2 (AI) colour — light red throughout the UI.
pub const COLOR_P2: Color = Color::LightRed;
/// Accent colour used for borders, titles, rosette foreground, and highlights.
pub const COLOR_ACCENT: Color = Color::Yellow;
/// Rosette square background tint.
pub const COLOR_ROSETTE_BG: Color = Color::Rgb(61, 43, 31);
/// Background for the currently selected (cursor) square.
pub const COLOR_SELECTED_BG: Color = Color::Rgb(30, 60, 30);
/// Background for the target (legal move destination) square.
pub const COLOR_TARGET_BG: Color = Color::Rgb(40, 20, 60);
/// Subdued text (subtitles, secondary labels).
pub const COLOR_DIM: Color = Color::DarkGray;
/// Mid-brightness text (descriptions, inactive labels).
pub const COLOR_SUB: Color = Color::Gray;

/// Formats a duration as `MM:SS`.
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
