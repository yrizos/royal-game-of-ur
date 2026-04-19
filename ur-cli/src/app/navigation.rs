use super::CURSOR_BEAR_OFF;

/// 2-D navigation grid: NAV_GRID[visual_row][col] → cursor_path_pos.
/// Visual row 0 = top of board (finish end); row 7 = bottom (entry end).
/// Col 0 = left (player private lane + virtual off-board slots).
/// Col 1 = right (shared middle column).
///
/// Board layout after vertical flip:
///   row 0: [step 13, step 12]
///   row 1: [step 14✦, step 11]
///   row 2: [BEAR_OFF, step 10]   ← H-gap: virtual slots on left
///   row 3: [pool (0), step 9]    ← H-gap
///   row 4: [step 1, step 8✦]
///   row 5: [step 2, step 7]
///   row 6: [step 3, step 6]
///   row 7: [step 4✦, step 5]
pub(crate) const NAV_GRID: [[usize; 2]; 8] = [
    [13, 12],
    [14, 11],
    [CURSOR_BEAR_OFF, 10],
    [0, 9],
    [1, 8],
    [2, 7],
    [3, 6],
    [4, 5],
];

/// Direction for 2-D cursor navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavDir {
    Up,
    Down,
    Left,
    Right,
}

/// Returns the (row, col) grid coordinates for a cursor position, or `None`
/// if `pos` is not represented in the grid.
fn cursor_to_grid(pos: usize) -> Option<(usize, usize)> {
    for (row, row_vals) in NAV_GRID.iter().enumerate() {
        for (col, &cell) in row_vals.iter().enumerate() {
            if cell == pos {
                return Some((row, col));
            }
        }
    }
    None
}

/// Returns the cursor position reached by moving one step in `dir` from `pos`.
/// UP/DOWN wrap across columns at the board edges.
pub fn nav_cursor(pos: usize, dir: NavDir) -> usize {
    let (row, col) = match cursor_to_grid(pos) {
        Some(p) => p,
        None => return pos,
    };
    let (new_row, new_col) = match dir {
        NavDir::Left => (row, col.saturating_sub(1)),
        NavDir::Right => (row, (col + 1).min(1)),
        NavDir::Up => {
            if row > 0 {
                (row - 1, col)
            } else {
                (0, 1 - col) // wrap: swap column at top row
            }
        }
        NavDir::Down => {
            if row < 7 {
                (row + 1, col)
            } else {
                (7, 1 - col) // wrap: swap column at bottom row
            }
        }
    };
    NAV_GRID[new_row][new_col]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_pos1_right_goes_to_pos8() {
        assert_eq!(nav_cursor(1, NavDir::Right), 8);
    }

    #[test]
    fn test_nav_pos9_left_goes_to_pool() {
        assert_eq!(nav_cursor(9, NavDir::Left), 0);
    }

    #[test]
    fn test_nav_pos10_left_goes_to_bear_off() {
        assert_eq!(nav_cursor(10, NavDir::Left), CURSOR_BEAR_OFF);
    }

    #[test]
    fn test_nav_pos4_right_goes_to_pos5() {
        assert_eq!(nav_cursor(4, NavDir::Right), 5);
    }

    #[test]
    fn test_nav_pos4_down_goes_to_pos5() {
        assert_eq!(nav_cursor(4, NavDir::Down), 5);
    }

    #[test]
    fn test_nav_pos12_left_goes_to_pos13() {
        assert_eq!(nav_cursor(12, NavDir::Left), 13);
    }

    #[test]
    fn test_nav_pos12_up_goes_to_pos13() {
        assert_eq!(nav_cursor(12, NavDir::Up), 13);
    }

    #[test]
    fn test_nav_pool_right_goes_to_pos9() {
        assert_eq!(nav_cursor(0, NavDir::Right), 9);
    }

    #[test]
    fn test_nav_bear_off_right_goes_to_pos10() {
        assert_eq!(nav_cursor(CURSOR_BEAR_OFF, NavDir::Right), 10);
    }

    #[test]
    fn test_nav_bear_off_down_goes_to_pool() {
        assert_eq!(nav_cursor(CURSOR_BEAR_OFF, NavDir::Down), 0);
    }

    #[test]
    fn test_nav_pool_up_goes_to_bear_off() {
        assert_eq!(nav_cursor(0, NavDir::Up), CURSOR_BEAR_OFF);
    }

    #[test]
    fn test_nav_horizontal_pairs() {
        let pairs = [
            (13usize, 12usize),
            (14, 11),
            (CURSOR_BEAR_OFF, 10),
            (0, 9),
            (1, 8),
            (2, 7),
            (3, 6),
            (4, 5),
        ];
        for (left, right) in pairs {
            assert_eq!(nav_cursor(left, NavDir::Right), right, "{left} right");
            assert_eq!(nav_cursor(right, NavDir::Left), left, "{right} left");
        }
    }

    #[test]
    fn test_nav_left_from_leftmost_stays() {
        for pos in [13, 14, CURSOR_BEAR_OFF, 0, 1, 2, 3, 4] {
            assert_eq!(
                nav_cursor(pos, NavDir::Left),
                pos,
                "pos {pos} already leftmost"
            );
        }
    }

    #[test]
    fn test_nav_right_from_rightmost_stays() {
        for pos in [12, 11, 10, 9, 8, 7, 6, 5] {
            assert_eq!(
                nav_cursor(pos, NavDir::Right),
                pos,
                "pos {pos} already rightmost"
            );
        }
    }

    #[test]
    fn test_nav_up_left_column() {
        let left_col = [13, 14, CURSOR_BEAR_OFF, 0, 1, 2, 3, 4];
        for i in 1..8 {
            assert_eq!(
                nav_cursor(left_col[i], NavDir::Up),
                left_col[i - 1],
                "up from left_col[{i}]={}",
                left_col[i]
            );
        }
    }

    #[test]
    fn test_nav_down_left_column() {
        let left_col = [13, 14, CURSOR_BEAR_OFF, 0, 1, 2, 3, 4];
        for i in 0..7 {
            assert_eq!(
                nav_cursor(left_col[i], NavDir::Down),
                left_col[i + 1],
                "down from left_col[{i}]={}",
                left_col[i]
            );
        }
    }

    #[test]
    fn test_nav_up_right_column() {
        let right_col = [12, 11, 10, 9, 8, 7, 6, 5];
        for i in 1..8 {
            assert_eq!(
                nav_cursor(right_col[i], NavDir::Up),
                right_col[i - 1],
                "up from right_col[{i}]={}",
                right_col[i]
            );
        }
    }

    #[test]
    fn test_nav_down_right_column() {
        let right_col = [12, 11, 10, 9, 8, 7, 6, 5];
        for i in 0..7 {
            assert_eq!(
                nav_cursor(right_col[i], NavDir::Down),
                right_col[i + 1],
                "down from right_col[{i}]={}",
                right_col[i]
            );
        }
    }

    #[test]
    fn test_nav_up_wraps_at_top_row() {
        assert_eq!(nav_cursor(13, NavDir::Up), 12, "step 13 up → step 12");
        assert_eq!(nav_cursor(12, NavDir::Up), 13, "step 12 up → step 13");
    }

    #[test]
    fn test_nav_down_wraps_at_bottom_row() {
        assert_eq!(nav_cursor(4, NavDir::Down), 5, "step 4 down → step 5");
        assert_eq!(nav_cursor(5, NavDir::Down), 4, "step 5 down → step 4");
    }
}
