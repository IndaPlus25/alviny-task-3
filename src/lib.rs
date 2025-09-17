/*****************************
*      CHESS GAME ENGINE     *
*      AUTHOR: alviny        *
*****************************/

pub struct Game{
    fen: String,
    board: Board,
}
impl Game {
    pub fn new() -> Game {
        Game {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            board: parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()),
        }
    }
    pub fn state_based_action_check() {
        // Chess SBAs: 
        // When 2 pieces share a coordinate, the piece of NAP is removed from the game.
        // When the AP has no legal moves and is in check, NAP wins the game.
        // When the AP has no legal moves and is not in check, the game is a draw.
    }
    //pub fn 
}
pub struct Board {
    board_state: Vec<Vec<char>>,
    active_player: char,
    castling_availability: String, 
    //This value is the power set of string "KQkq" and 
    // represents which castling moves are available. 
    // Castling not implemented yet.

    en_passant_square: String, 
    // This value represents whether or not en passant is available, 
    // and if so, the square to which the capturing pawn will move. Otherwise, 
    // the value will be "-".
    // En passant not implemented yet.

    halfmove_counter: u8,
    // This counter increments for every move made without a capture
    // or a pawn move. Otherwise, it resets. 
    // When it reaches 100, the game is a draw.
    turn_counter: u8,
    // This counter increments by one every time Black makes a move.
}


fn parse_fen(fen: String) -> Board {
    let mut fen_vec = fen
        .split(' ')
        .collect::<Vec<&str>>();
    let mut board_state_str = fen_vec[0];


}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
