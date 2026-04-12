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
        let mut valid_squares = Vec::with_capacity(20);
        for row in 0u8..3 {
            for col in 0u8..8 {
                if row != 1 && (col == 4 || col == 5) {
                    continue; // these squares don't exist
                }
                valid_squares.push(Square::new(row, col));
            }
        }
        let rosettes = vec![
            Square::new(0, 0),
            Square::new(0, 6),
            Square::new(1, 3),
            Square::new(2, 0),
            Square::new(2, 6),
        ];
        Self { valid_squares, rosettes }
    }

    /// Returns true if the given square exists on this board.
    pub fn is_valid(&self, sq: Square) -> bool {
        self.valid_squares.contains(&sq)
    }

    /// Returns true if the given square is a rosette.
    pub fn is_rosette(&self, sq: Square) -> bool {
        self.rosettes.contains(&sq)
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
    use super::*;

    fn player1_path() -> Path {
        Path::new(vec![
            Square::new(2, 3), Square::new(2, 2), Square::new(2, 1), Square::new(2, 0),
            Square::new(1, 0), Square::new(1, 1), Square::new(1, 2), Square::new(1, 3),
            Square::new(1, 4), Square::new(1, 5), Square::new(1, 6), Square::new(1, 7),
            Square::new(2, 7), Square::new(2, 6),
        ])
    }

    fn player2_path() -> Path {
        Path::new(vec![
            Square::new(0, 3), Square::new(0, 2), Square::new(0, 1), Square::new(0, 0),
            Square::new(1, 0), Square::new(1, 1), Square::new(1, 2), Square::new(1, 3),
            Square::new(1, 4), Square::new(1, 5), Square::new(1, 6), Square::new(1, 7),
            Square::new(0, 7), Square::new(0, 6),
        ])
    }

    #[test]
    fn test_path_player1_covers_correct_squares() {
        let path = player1_path();
        assert_eq!(path.len(), 14);
        // Private leg: row 2 descending to col 0
        assert_eq!(path.get(0), Some(Square::new(2, 3)));
        assert_eq!(path.get(1), Some(Square::new(2, 2)));
        assert_eq!(path.get(2), Some(Square::new(2, 1)));
        assert_eq!(path.get(3), Some(Square::new(2, 0)));
        // Shared middle row: col 0 through 7
        for col in 0u8..8 {
            assert_eq!(path.get(4 + col as usize), Some(Square::new(1, col)));
        }
        // Exit leg: row 2 cols 7 then 6
        assert_eq!(path.get(12), Some(Square::new(2, 7)));
        assert_eq!(path.get(13), Some(Square::new(2, 6)));
    }

    #[test]
    fn test_path_player2_covers_correct_squares() {
        let path = player2_path();
        assert_eq!(path.len(), 14);
        // Private leg: row 0 descending to col 0
        assert_eq!(path.get(0), Some(Square::new(0, 3)));
        assert_eq!(path.get(1), Some(Square::new(0, 2)));
        assert_eq!(path.get(2), Some(Square::new(0, 1)));
        assert_eq!(path.get(3), Some(Square::new(0, 0)));
        // Shared middle row: col 0 through 7
        for col in 0u8..8 {
            assert_eq!(path.get(4 + col as usize), Some(Square::new(1, col)));
        }
        // Exit leg: row 0 cols 7 then 6
        assert_eq!(path.get(12), Some(Square::new(0, 7)));
        assert_eq!(path.get(13), Some(Square::new(0, 6)));
    }

    #[test]
    fn test_shared_row_is_columns_0_through_7() {
        let p1 = player1_path();
        let p2 = player2_path();
        for col in 0u8..8 {
            let sq = Square::new(1, col);
            assert!(
                p1.index_of(sq).is_some(),
                "Player1 path missing shared square (1,{})", col
            );
            assert!(
                p2.index_of(sq).is_some(),
                "Player2 path missing shared square (1,{})", col
            );
        }
    }

    #[test]
    fn test_rosette_positions() {
        let shape = BoardShape::finkel();
        let expected_rosettes = vec![
            Square::new(0, 0),
            Square::new(0, 6),
            Square::new(1, 3),
            Square::new(2, 0),
            Square::new(2, 6),
        ];
        for sq in &expected_rosettes {
            assert!(shape.is_rosette(*sq), "expected rosette at ({},{})", sq.row, sq.col);
        }
        // Verify no extra rosettes
        assert_eq!(shape.rosettes().len(), 5);
    }

    #[test]
    fn test_invalid_squares_row0_cols_4_5() {
        let shape = BoardShape::finkel();
        assert!(!shape.is_valid(Square::new(0, 4)), "(0,4) should be invalid");
        assert!(!shape.is_valid(Square::new(0, 5)), "(0,5) should be invalid");
    }

    #[test]
    fn test_invalid_squares_row2_cols_4_5() {
        let shape = BoardShape::finkel();
        assert!(!shape.is_valid(Square::new(2, 4)), "(2,4) should be invalid");
        assert!(!shape.is_valid(Square::new(2, 5)), "(2,5) should be invalid");
    }

    #[test]
    fn test_valid_square_count_is_20() {
        let shape = BoardShape::finkel();
        assert_eq!(shape.valid_squares().len(), 20);
    }

    #[test]
    fn test_path_index_of_roundtrip() {
        let path = player1_path();
        for (i, &sq) in path.squares().iter().enumerate() {
            assert_eq!(path.index_of(sq), Some(i));
            assert_eq!(path.get(path.index_of(sq).unwrap()), Some(sq));
        }
    }
}
