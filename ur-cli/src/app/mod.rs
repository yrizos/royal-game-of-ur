mod gameplay;
mod handlers;
pub mod navigation;

pub use navigation::NavDir;

use crate::animation::Animation;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;
use ur_core::{
    dice::Dice,
    state::{GameState, Move},
};

/// A single entry in the game event log.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Which player caused this event. `None` for system messages.
    pub player: Option<ur_core::player::Player>,
    pub text: String,
}

/// Which screen is currently active.
#[derive(Debug)]
pub enum Screen {
    Title,
    DifficultySelect {
        selected: usize,
    },
    DiceOff {
        state: DiceOffState,
    },
    Game,
    /// Pause overlay shown when pressing Esc during gameplay.
    /// `selected`: 0 = Resume, 1 = How to Play, 2 = Quit
    PauseMenu {
        selected: usize,
    },
    /// Rules + key-binding help screen.
    /// `from_game`: true when reached via the pause menu; Back returns to PauseMenu.
    /// false when reached from the title screen; Back returns to Title.
    Help {
        from_game: bool,
    },
    GameOver,
}

/// State for the first-player dice-off animation.
#[derive(Debug)]
pub struct DiceOffState {
    pub p1_frames: u32,
    pub p2_frames: u32,
    pub p1_final: Dice,
    pub p2_final: Dice,
    pub p1_display: Dice,
    pub p2_display: Dice,
    pub winner: Option<ur_core::player::Player>,
    pub acknowledged: bool,
}

/// Number of animation ticks for the dice-off countdown (≈1.5 s at 12 fps).
const DICE_OFF_ANIMATION_FRAMES: u32 = 18;

/// Number of animation ticks for a dice roll animation (≈1 s at 30 fps).
const DICE_ROLL_ANIMATION_FRAMES: u32 = 30;

/// Delay in ms before the dice roll animation fires automatically.
const AUTO_ROLL_DELAY_MS: u64 = 300;
/// How long (ms) to display a no-moves result before forfeiting.
const FORFEIT_DISPLAY_MS: u64 = 1000;

/// Difficulty level maps to expectiminimax search depth.
pub const DIFFICULTIES: [(&str, u32); 3] = [("Easy", 2), ("Medium", 4), ("Hard", 6)];

/// Cursor position representing the off-board "bearing off" (scored) slot.
pub const CURSOR_BEAR_OFF: usize = 15;

/// Game statistics accumulated during play.
#[derive(Debug, Default)]
pub struct GameStats {
    pub moves: u32,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub captures: [u32; 2],
}

/// Top-level application state.
pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub difficulty: u32,
    pub game_state: Option<GameState>,
    pub dice_roll: Option<Dice>,
    pub legal_moves: Vec<Move>,
    /// Cursor position along the current player's path.
    /// 0 = unplayed-pieces pool; k = path square at index k-1 (1-based).
    pub cursor_path_pos: usize,
    pub log: Vec<LogEntry>,
    pub log_visible: bool,
    pub stats: GameStats,
    pub rng: SmallRng,
    pub title_selected: usize,
    pub animation: Option<Animation>,
    pub ai_thinking: bool,
    pub ai_spinner_frame: u32,
    pub ai_receiver: Option<std::sync::mpsc::Receiver<ur_core::state::Move>>,
    /// Scroll offset for the help screen (lines scrolled down).
    pub help_scroll: u16,
    /// True when a dice roll should fire automatically as soon as conditions allow.
    pub pending_roll: bool,
    /// Earliest time at which the auto-roll may fire (None = fire immediately).
    pub roll_after: Option<std::time::Instant>,
    /// When set, the no-moves forfeit fires at this time.
    pub forfeit_after: Option<std::time::Instant>,
    /// True when the auto-roll is a rosette re-roll (skips the normal delay).
    pub rosette_reroll: bool,
    /// Last dice roll per player (index 0 = P1, 1 = P2). Kept for the inactive panel.
    pub last_roll: [Option<Dice>; 2],
    /// Last notable event per player (capture / rosette / score). Shown below dice.
    pub last_event: [Option<String>; 2],
    /// Per-player turn history: up to 3 past turns, each a list of event strings.
    pub turn_log: [Vec<Vec<String>>; 2],
    /// Monotonically increasing counter, incremented every animation tick.
    pub frame_count: u32,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title,
            should_quit: false,
            difficulty: 4,
            game_state: None,
            dice_roll: None,
            legal_moves: Vec::new(),
            cursor_path_pos: 0,
            log: Vec::new(),
            log_visible: false,
            stats: GameStats::default(),
            rng: SmallRng::from_entropy(),
            title_selected: 0,
            animation: None,
            ai_thinking: false,
            ai_spinner_frame: 0,
            ai_receiver: None,
            help_scroll: 0,
            pending_roll: false,
            roll_after: None,
            forfeit_after: None,
            rosette_reroll: false,
            last_roll: [None, None],
            last_event: [None, None],
            turn_log: [vec![vec![]], vec![vec![]]],
            frame_count: 0,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_uses_you_label_for_player1() {
        let entry = LogEntry {
            player: Some(ur_core::player::Player::Player1),
            text: "rolled 3".to_string(),
        };
        assert_eq!(entry.player, Some(ur_core::player::Player::Player1));
        assert!(entry.text.contains("rolled 3"));
    }

    #[test]
    fn test_log_entry_uses_ai_label_for_player2() {
        let entry = LogEntry {
            player: Some(ur_core::player::Player::Player2),
            text: "rolled 2".to_string(),
        };
        assert_eq!(entry.player, Some(ur_core::player::Player::Player2));
    }
}
