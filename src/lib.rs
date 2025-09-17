/*****************************
*      CHESS GAME ENGINE     *
*      AUTHOR: alviny        *
*****************************/
use std::{fmt, iter::Enumerate};

//#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Game {
    fen: String,
    board: Board,
}
impl Game {
    pub fn new() -> Game {
        Game {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            board: parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
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
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
pub struct Board {
    board_state: Vec<Vec<char>>, 
    // Represents the board. Pieces are represented by their FEN notation (capital for white, lowercase for black)
    // Blank squares are represented by "-"
    active_player: char, // "W" or "B"
    castling_availability: String, // TODO: change this to String
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
    turn_counter: u64,
    // This counter increments by one every time Black makes a move.
}
impl Board{
    
}
impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.board_state)
    }
}
fn parse_fen(fen: &str) -> Board {
    let fen_vec = fen
        .split(' ')
        .collect::<Vec<&str>>();
    //Split the FEN into its constituent parts
 
    let board_state_vec = fen_vec[0]
        .split('/')
        .collect::<Vec<&str>>();
    let mut row = vec![];
    let mut board_state = vec![]; 
    for single_row in board_state_vec {
        for character in single_row.chars() {
            //assuming valid FEN (only characters and numbers)
            const RADIX: u32 = 10;
            if character.is_numeric() {
                for i in 0..character
                    .to_digit(RADIX)
                    .unwrap() {
                        row.push('*');
                    }
            } else {
                row.push(character);
            }
        }
        board_state.push(row.clone());
        row.retain(|_x| false); // empty the vector
    }
    //Parse the board state part of the FEN into a 2d nested Vec

    Board{
        board_state,
        active_player: fen_vec[1].chars().next().expect("string is empty"),
        castling_availability: fen_vec[2].to_string(),
        en_passant_square: fen_vec[3].to_string(),
        halfmove_counter: fen_vec[4]
            .parse::<u8>()
            .expect("not possible to convert to u8"),
        turn_counter: fen_vec[5]
            .parse::<u64>()
            .expect("not possible to convert to u64"),
    }
    //Then feed the rest directly into the cosntructor

}












/* Unit test example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
*/
