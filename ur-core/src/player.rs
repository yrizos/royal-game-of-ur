/// One of the two players.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    /// Returns the opposing player.
    pub fn opponent(self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    /// Returns the zero-based index used for array lookup (Player1 = 0, Player2 = 1).
    pub fn index(self) -> usize {
        match self {
            Player::Player1 => 0,
            Player::Player2 => 1,
        }
    }
}

/// A single game piece, identified by its owner and a 0-based index (0–6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    /// Which player owns this piece.
    pub player: Player,
    /// Zero-based index distinguishing this piece from the player's other pieces (0–6).
    pub index: u8,
}

impl Piece {
    /// Creates a piece owned by `player` with the given zero-based `index`.
    pub fn new(player: Player, index: u8) -> Self {
        Self { player, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player1_opponent_is_player2() {
        assert_eq!(Player::Player1.opponent(), Player::Player2);
    }

    #[test]
    fn test_player2_opponent_is_player1() {
        assert_eq!(Player::Player2.opponent(), Player::Player1);
    }

    #[test]
    fn test_opponent_is_involution() {
        assert_eq!(Player::Player1.opponent().opponent(), Player::Player1);
        assert_eq!(Player::Player2.opponent().opponent(), Player::Player2);
    }

    #[test]
    fn test_player_index_player1_is_0() {
        assert_eq!(Player::Player1.index(), 0);
    }

    #[test]
    fn test_player_index_player2_is_1() {
        assert_eq!(Player::Player2.index(), 1);
    }
}
