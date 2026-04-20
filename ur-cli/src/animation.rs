use crate::app::App;
use ur_core::{board::Square, dice::Dice};

/// Current animation state.
#[derive(Debug)]
pub enum Animation {
    /// Dice rolling — cycles display until frames_remaining hits 0.
    DiceRoll {
        frames_remaining: u32,
        final_value: Dice,
        display: Dice,
    },
    /// Piece moving square by square along a path.
    PieceMove {
        remaining: Vec<Square>,
        current_idx: usize,
        frames_per_step: u32,
        frames_this_step: u32,
        is_player1: bool,
    },
    /// Captured piece flashing before disappearing.
    CaptureFlash {
        square: Square,
        frames_remaining: u32,
    },
    /// Animation finished — caller should clear this.
    Done,
}

/// Tick one animation frame. Transitions to `Done` when complete.
pub fn tick_animation(anim: &mut Animation) {
    match anim {
        Animation::DiceRoll {
            frames_remaining,
            final_value,
            display,
        } => {
            if *frames_remaining <= 1 {
                *display = *final_value;
                *anim = Animation::Done;
            } else {
                *frames_remaining -= 1;
                display.cycle();
            }
        }
        Animation::PieceMove {
            remaining,
            current_idx,
            frames_per_step,
            frames_this_step,
            ..
        } => {
            if *frames_this_step > 0 {
                *frames_this_step -= 1;
            } else if *current_idx >= remaining.len() {
                *anim = Animation::Done;
            } else {
                *current_idx += 1;
                *frames_this_step = *frames_per_step;
            }
        }
        Animation::CaptureFlash {
            frames_remaining, ..
        } => {
            if *frames_remaining <= 1 {
                *anim = Animation::Done;
            } else {
                *frames_remaining -= 1;
            }
        }
        Animation::Done => {}
    }
}

/// Advance app animations and handle post-animation transitions.
pub fn tick(app: &mut App) {
    app.frame_count = app.frame_count.wrapping_add(1);

    // Advance active animation
    if let Some(anim) = &mut app.animation {
        tick_animation(anim);
        if matches!(anim, Animation::Done) {
            app.animation = None;
            app.on_animation_done();
        }
    }

    // Advance DiceOff screen animation
    if let crate::app::Screen::DiceOff { state } = &mut app.screen {
        fn tick_die(
            frames: &mut u32,
            display: &mut ur_core::dice::Dice,
            final_val: ur_core::dice::Dice,
        ) {
            if *frames > 0 {
                *frames -= 1;
                display.cycle();
                if *frames == 0 {
                    *display = final_val;
                }
            }
        }
        tick_die(&mut state.p1_frames, &mut state.p1_display, state.p1_final);
        tick_die(&mut state.p2_frames, &mut state.p2_display, state.p2_final);
        // Determine winner once both animations complete
        if state.p1_frames == 0 && state.p2_frames == 0 && state.winner.is_none() {
            use ur_core::player::Player;
            state.winner = match state.p1_final.value().cmp(&state.p2_final.value()) {
                std::cmp::Ordering::Greater => Some(Player::Player1),
                std::cmp::Ordering::Less => Some(Player::Player2),
                std::cmp::Ordering::Equal => None, // tie — schedule re-roll below
            };

            // On a tie, reset and re-roll after the UI has had a chance to show "Tie"
            if state.winner.is_none() {
                const FRAMES: u32 = 18;
                state.p1_frames = FRAMES;
                state.p2_frames = FRAMES;
                state.p1_final = ur_core::dice::Dice::roll(&mut app.rng);
                state.p2_final = ur_core::dice::Dice::roll(&mut app.rng);
                state.p1_display = Dice::new(0).unwrap();
                state.p2_display = Dice::new(0).unwrap();
            }
        }
    }

    // AI spinner
    if app.ai_thinking {
        app.ai_spinner_frame = (app.ai_spinner_frame + 1) % 4;
        app.poll_ai_move();
    }

    // Auto-roll and forfeit delay
    app.tick_auto_roll();
    app.tick_forfeit_delay();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ur_core::dice::Dice;

    #[test]
    fn test_dice_animation_counts_down() {
        let mut anim = Animation::DiceRoll {
            frames_remaining: 5,
            final_value: Dice::new(3).unwrap(),
            display: Dice::new(0).unwrap(),
        };
        tick_animation(&mut anim);
        match anim {
            Animation::DiceRoll {
                frames_remaining, ..
            } => assert_eq!(frames_remaining, 4),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_dice_animation_finishes_at_zero() {
        let mut anim = Animation::DiceRoll {
            frames_remaining: 1,
            final_value: Dice::new(3).unwrap(),
            display: Dice::new(0).unwrap(),
        };
        tick_animation(&mut anim);
        assert!(matches!(anim, Animation::Done));
    }

    #[test]
    fn test_capture_flash_counts_down() {
        let mut anim = Animation::CaptureFlash {
            square: ur_core::board::Square::new(1, 0),
            frames_remaining: 3,
        };
        tick_animation(&mut anim);
        match anim {
            Animation::CaptureFlash {
                frames_remaining, ..
            } => assert_eq!(frames_remaining, 2),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_capture_flash_finishes_at_one() {
        let mut anim = Animation::CaptureFlash {
            square: ur_core::board::Square::new(1, 0),
            frames_remaining: 1,
        };
        tick_animation(&mut anim);
        assert!(matches!(anim, Animation::Done));
    }

    #[test]
    fn test_piece_move_advances_step_when_frame_reaches_zero() {
        let sq0 = ur_core::board::Square::new(1, 0);
        let sq1 = ur_core::board::Square::new(1, 1);
        let mut anim = Animation::PieceMove {
            remaining: vec![sq0, sq1],
            current_idx: 0,
            frames_per_step: 2,
            frames_this_step: 0,
            is_player1: true,
        };
        tick_animation(&mut anim);
        match &anim {
            Animation::PieceMove {
                current_idx,
                frames_this_step,
                ..
            } => {
                assert_eq!(*current_idx, 1, "index should advance past first square");
                assert_eq!(
                    *frames_this_step, 2,
                    "frames_this_step should reset to frames_per_step"
                );
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_piece_move_completes_when_remaining_empty_and_frame_zero() {
        let mut anim = Animation::PieceMove {
            remaining: vec![],
            current_idx: 0,
            frames_per_step: 3,
            frames_this_step: 0,
            is_player1: true,
        };
        tick_animation(&mut anim);
        assert!(matches!(anim, Animation::Done));
    }

    #[test]
    fn test_piece_move_counts_down_frame_before_advancing() {
        let sq = ur_core::board::Square::new(1, 0);
        let mut anim = Animation::PieceMove {
            remaining: vec![sq],
            current_idx: 0,
            frames_per_step: 3,
            frames_this_step: 3,
            is_player1: false,
        };
        tick_animation(&mut anim);
        match &anim {
            Animation::PieceMove {
                current_idx,
                frames_this_step,
                ..
            } => {
                assert_eq!(*current_idx, 0, "index should not advance yet");
                assert_eq!(*frames_this_step, 2);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_done_is_noop() {
        let mut anim = Animation::Done;
        tick_animation(&mut anim);
        assert!(matches!(anim, Animation::Done));
    }

    #[test]
    fn test_tick_calls_tick_auto_roll() {
        use crate::app::{App, Screen};
        use ur_core::state::{GameRules, GameState};

        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(GameState::new(&GameRules::finkel()));
        // Set pending_roll with an already-elapsed deadline
        app.pending_roll = true;
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));

        tick(&mut app);

        assert!(
            matches!(app.animation, Some(Animation::DiceRoll { .. })),
            "tick() must fire auto-roll when pending_roll is set and deadline is past"
        );
    }

    #[test]
    fn test_tick_calls_tick_forfeit_delay() {
        use crate::app::{App, Screen};
        use ur_core::{
            dice::Dice,
            state::{GameRules, GameState},
        };

        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(GameState::new(&GameRules::finkel()));
        app.dice_roll = Some(Dice::new(0).unwrap());
        app.forfeit_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));

        tick(&mut app);

        assert!(
            app.forfeit_after.is_none(),
            "tick() must fire forfeit when deadline is past"
        );
        // start_ai_turn() always sets dice_roll to the AI's roll (even when
        // moves are empty), so dice_roll is Some after the forfeit fires.
        assert!(
            app.dice_roll.is_some(),
            "AI turn should have started after forfeit"
        );
    }
}
