/// One of the two players.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    /// Returns the opposing player.
    pub fn opponent(self) -> Player {
        todo!()
    }

    /// Returns the zero-based index used for array lookup (Player1 = 0, Player2 = 1).
    pub fn index(self) -> usize {
        todo!()
    }
}

/// A single game piece, identified by its owner and a 0-based index (0–6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub player: Player,
    pub index: u8,
}

impl Piece {
    pub fn new(player: Player, index: u8) -> Self {
        Self { player, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player1_opponent_is_player2() {
        todo!()
    }

    #[test]
    fn test_player2_opponent_is_player1() {
        todo!()
    }

    #[test]
    fn test_opponent_is_involution() {
        // p.opponent().opponent() == p for both players
        todo!()
    }

    #[test]
    fn test_player_index_player1_is_0() {
        todo!()
    }

    #[test]
    fn test_player_index_player2_is_1() {
        todo!()
    }
}
