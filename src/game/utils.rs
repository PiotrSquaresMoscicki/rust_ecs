use super::components::{GRID_HEIGHT, GRID_WIDTH};
use std::collections::HashSet;

/// Calculate the next move towards the target while avoiding obstacles
pub fn calculate_next_move(
    current: (i32, i32),
    target: (i32, i32),
    obstacles: &HashSet<(i32, i32)>,
) -> (i32, i32) {
    let (cx, cy) = current;
    let (tx, ty) = target;

    // Calculate direction
    let dx = if tx > cx {
        1
    } else if tx < cx {
        -1
    } else {
        0
    };
    let dy = if ty > cy {
        1
    } else if ty < cy {
        -1
    } else {
        0
    };

    // Try diagonal movement first
    let diagonal = (cx + dx, cy + dy);
    if !obstacles.contains(&diagonal) && is_valid_position(diagonal) {
        return diagonal;
    }

    // Try horizontal movement
    if dx != 0 {
        let horizontal = (cx + dx, cy);
        if !obstacles.contains(&horizontal) && is_valid_position(horizontal) {
            return horizontal;
        }
    }

    // Try vertical movement
    if dy != 0 {
        let vertical = (cx, cy + dy);
        if !obstacles.contains(&vertical) && is_valid_position(vertical) {
            return vertical;
        }
    }

    // Can't move, stay in place
    current
}

/// Check if a position is within the grid bounds
pub fn is_valid_position(pos: (i32, i32)) -> bool {
    pos.0 >= 0 && pos.0 < GRID_WIDTH && pos.1 >= 0 && pos.1 < GRID_HEIGHT
}

/// Check if two positions are adjacent (including diagonally)
pub fn is_adjacent(pos1: (i32, i32), pos2: (i32, i32)) -> bool {
    let dx = (pos1.0 - pos2.0).abs();
    let dy = (pos1.1 - pos2.1).abs();
    dx <= 1 && dy <= 1 && !(dx == 0 && dy == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_position() {
        assert!(is_valid_position((0, 0)));
        assert!(is_valid_position((29, 14))); // Max position for 30x15 grid
        assert!(!is_valid_position((-1, 0)));
        assert!(!is_valid_position((30, 0))); // Out of bounds width
        assert!(!is_valid_position((0, -1)));
        assert!(!is_valid_position((0, 15))); // Out of bounds height
    }

    #[test]
    fn test_is_adjacent() {
        assert!(is_adjacent((1, 1), (1, 2))); // vertical
        assert!(is_adjacent((1, 1), (2, 1))); // horizontal
        assert!(is_adjacent((1, 1), (2, 2))); // diagonal
        assert!(is_adjacent((1, 1), (0, 0))); // diagonal
        assert!(!is_adjacent((1, 1), (1, 1))); // same position
        assert!(!is_adjacent((1, 1), (3, 3))); // too far
    }

    #[test]
    fn test_calculate_next_move() {
        let obstacles = HashSet::new();

        // Test direct movement
        assert_eq!(calculate_next_move((0, 0), (2, 2), &obstacles), (1, 1));
        assert_eq!(calculate_next_move((5, 5), (3, 3), &obstacles), (4, 4));

        // Test with obstacle
        let mut obstacles_with_block = HashSet::new();
        obstacles_with_block.insert((1, 1));
        // Should move horizontally or vertically when diagonal is blocked
        let next = calculate_next_move((0, 0), (2, 2), &obstacles_with_block);
        assert!(next == (1, 0) || next == (0, 1));
    }
}
