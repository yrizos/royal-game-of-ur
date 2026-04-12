/// A position on the board identified by row and column.
///
/// Valid squares follow the Finkel 3×8 layout with four removed squares:
/// row 0 and row 2 have no columns 4 or 5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Square {
    pub row: u8,
    pub col: u8,
}

impl Square {
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }
}

/// Defines which squares exist on the board and which are rosettes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardShape {
    valid_squares: Vec<Square>,
    rosettes: Vec<Square>,
}

impl BoardShape {
    /// Returns the standard 20-square Finkel board shape.
    ///
    /// Valid squares: row 0 cols 0-3 and 6-7; row 1 cols 0-7; row 2 cols 0-3 and 6-7.
    /// Rosettes: (0,0), (0,6), (1,3), (2,0), (2,6).
    pub fn finkel() -> Self {
        todo!()
    }

    /// Returns true if the given square exists on this board.
    pub fn is_valid(&self, _sq: Square) -> bool {
        todo!()
    }

    /// Returns true if the given square is a rosette.
    pub fn is_rosette(&self, _sq: Square) -> bool {
        todo!()
    }

    pub fn valid_squares(&self) -> &[Square] {
        &self.valid_squares
    }

    pub fn rosettes(&self) -> &[Square] {
        &self.rosettes
    }
}

/// An ordered sequence of squares defining a player's route from entry to exit.
///
/// Does not include the logical off-board entry or exit positions; those are
/// handled by `PieceLocation` in the state module.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    squares: Vec<Square>,
}

impl Path {
    pub fn new(squares: Vec<Square>) -> Self {
        Self { squares }
    }

    pub fn squares(&self) -> &[Square] {
        &self.squares
    }

    pub fn len(&self) -> usize {
        self.squares.len()
    }

    pub fn is_empty(&self) -> bool {
        self.squares.is_empty()
    }

    /// Returns the square at position `index` along the path, if it exists.
    pub fn get(&self, index: usize) -> Option<Square> {
        self.squares.get(index).copied()
    }

    /// Returns the index of `sq` in this path, if present.
    pub fn index_of(&self, sq: Square) -> Option<usize> {
        self.squares.iter().position(|&s| s == sq)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_path_player1_covers_correct_squares() {
        // Player1 path (14 squares):
        // (2,3)->(2,2)->(2,1)->(2,0)->(1,0)->(1,1)->(1,2)->(1,3)->
        // (1,4)->(1,5)->(1,6)->(1,7)->(2,7)->(2,6)
        todo!()
    }

    #[test]
    fn test_path_player2_covers_correct_squares() {
        // Player2 path (14 squares):
        // (0,3)->(0,2)->(0,1)->(0,0)->(1,0)->(1,1)->(1,2)->(1,3)->
        // (1,4)->(1,5)->(1,6)->(1,7)->(0,7)->(0,6)
        todo!()
    }

    #[test]
    fn test_shared_row_is_columns_0_through_7() {
        // Row 1, columns 0-7 must appear on both Player1 and Player2 paths
        todo!()
    }

    #[test]
    fn test_rosette_positions() {
        // Rosettes at exactly: (0,0), (0,6), (1,3), (2,0), (2,6)
        todo!()
    }

    #[test]
    fn test_invalid_squares_row0_cols_4_5() {
        // BoardShape::finkel() must report (0,4) and (0,5) as invalid
        todo!()
    }

    #[test]
    fn test_invalid_squares_row2_cols_4_5() {
        // BoardShape::finkel() must report (2,4) and (2,5) as invalid
        todo!()
    }

    #[test]
    fn test_valid_square_count_is_20() {
        // The Finkel board has exactly 20 valid squares
        todo!()
    }

    #[test]
    fn test_path_index_of_roundtrip() {
        // path.get(path.index_of(sq).unwrap()) == Some(sq) for every sq in path
        todo!()
    }
}
