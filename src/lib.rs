/*****************************
*      CHESS GAME ENGINE     *
*      AUTHOR: alviny        *
*****************************/

// This library uses algebraic notation. Read more here: 
// https://en.wikipedia.org/wiki/Algebraic_notation_(chess)#Naming_the_pieces

// This library uses FEN notation. Read more here:
// https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation

use std::{fmt::{self, format}, iter::Enumerate};
use std::collections::HashMap;


/*****************************
*       HELPER FUNCTIONS     *
*       BEGIN HERE           *
*****************************/

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
                for _i in 0..character
                    .to_digit(RADIX)
                    .expect("Could not convert char to int") {
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
            .parse::<i32>()
            .expect("not possible to convert to i32"),
        turn_counter: fen_vec[5]
            .parse::<u64>()
            .expect("not possible to convert to u64"),
    }
    //Then feed the rest directly into the cosntructor

} // Creates a Board struct from any given FEN. Inverse function to generate_fen()

fn generate_fen(board: Board) -> String {
    let mut fen = String::new();
    let mut fen_row = String::new();
    let mut empty_squares: i32 = 0;
    for row in board.board_state {
        for char in row {
            if char == '*' {
                empty_squares += 1;
            } else {
                if empty_squares > 0 {
                    fen_row = format!("{}{}{}", fen_row, empty_squares, char);
                    empty_squares = 0;
                } else {
                    fen_row = format!("{}{}", fen_row, char);
                }
            }
        }
        if empty_squares > 0 {
            fen_row = format!("{}{}", fen_row, empty_squares);
            empty_squares = 0;
        } // In case the last few squares are empty, add them
        if fen.is_empty() {
            fen = fen_row;
        } else {
            fen = format!("{}/{}", fen, fen_row);
        }
    }
    return format!(
        "{} {} {} {} {} {}", 
        fen, 
        board.active_player, 
        board.castling_availability, 
        board.en_passant_square, 
        board.halfmove_counter, 
        board.turn_counter
    );
} // Creates a FEN from any given Board struct. Inverse function for parse_fen().

fn get_board_coords (algebraic_notation: String) -> Vec<i32> {
    let row_names = "hgfedcba".to_string();
    let row_number_index = row_names.chars().position(
        |x| 
            x == algebraic_notation
                .chars()
                .nth(0)
                .unwrap()
    ).expect("Unable to find row number");
    let row_number = i32::try_from(row_number_index).expect("Row number index too large");
    let col_number = algebraic_notation
                .chars()
                .nth(1)
                .unwrap()
                as i32;
    return vec![row_number, col_number - 1]
} // Generates Board.game_state coords from algebraic notation. [0~7, 0~7]. [0, 0] corresponds to h1, and [8, 8] is a8. 

fn get_algebaric_notation (coords: Vec<i32>) {
    let row_names = "hgfedcba".to_string();
}

fn get_piece(board: Board, coords: &Vec<i32>) -> char {
    return board.board_state[coords[0] as usize][coords[1] as usize]
} // Returns the piece on a given coordinate on the board.

fn set_piece(mut board: Board, coords: &Vec<i32>, piece: char) -> Board {
    board.board_state[coords[0] as usize][coords[1] as usize] = piece;
    return board
} // changes the given board coordinate to the given piece. 

fn move_piece(mut board: Board, source: String, target: String) -> Board { //Function assumes valid algebraic notation and valid move
    let source_coords = get_board_coords(source);
    let target_coords = get_board_coords(target);
    let mut increment_half_move_counter = true;
    let piece = get_piece(board, &source_coords);  // target square isn't empty => Capture 
    if get_piece(board.clone(), &target_coords) != '*' 
    || // OR
    piece.to_lowercase().to_string() == 'p'.to_string() { // the piece moved is a pawn
        increment_half_move_counter = false;
    }

    board
} // Moves a piece to a target square, and updates necessary counters on the board

fn get_piece_movements(coords: Vec<i32>, piece: &char) -> Vec<Vec<i32>> {
    todo!()
}
/*****************************
*       PUBLIC STRUCTS       *
*       BEGIN HERE           *
*****************************/

pub enum Pieces {
    Pawn, 
    Bishop,
    Knight, 
    Rook,
    Queen,
    King,
}
#[derive(Copy, Clone, PartialEq)] // I have no idea how to implement these traits (except for Debug), need to ask for / find walkthrough
pub struct Game {
    fen: String,
    board: Board,/*
    check_w: bool,
    check_b: bool,
    */
}
impl Game {
    pub fn new() -> Game {
        Game {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            board: parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            check_w: false,
            check_b: false,
        }
    }
    pub fn get_available_moves(board: Board, active_color: char) -> HashMap<Vec<i32>, Vec<Vec<i32>>>{
        let mut output = HashMap::new();
        for (y_pos, row) in board.board_state.iter().enumerate() {
            for (x_pos,piece) in row.iter().enumerate() {
                if active_color == 'w' {
                    if piece.is_ascii_uppercase() { // WHITE pieces are represented by uppercase letters
                        let coords: Vec<i32> = vec![i32::try_from(x_pos).unwrap(), i32::try_from(y_pos).unwrap()];
                        let movements = get_piece_movements(coords, &piece);
                        output.insert(coords, movements);
                    }
                } else if active_color == 'b' {
                    if piece.is_ascii_lowercase() { // black pieces are represented by lowercase letters
                        
                    }

                }
            }
        }
        output
    } // For any given color, finds pieces of that color. Returns a 
    //Hashmap of coords with pieces of that color, and available moves for each coordinate. 
    
    pub fn make_move(&mut self, source: String, target: String) { //Assuming both square and target are algebraic notation strings with length 2
        let available_moves = Self::get_available_moves(self.board, self.board.active_player);
        if available_moves.contains_key(&source) {
            self.board = move_piece(self.board, source, target);
            self.fen = generate_fen(self.board);
        }
    } // Make move if move is available for the active player, 
}
impl Default for Game {
    fn default() -> Self {
    Self::new()
    }
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Current FEN: {} \n Current board state: {:?}", self.fen, self.board)
    }
}
#[derive(/*Copy,*/ Clone, PartialEq)]
pub struct Board {
    board_state: Vec<Vec<char>>, 
    // Represents the board. Pieces are represented by their FEN notation (capital for white, lowercase for black)
    // Blank squares are represented by "-"
    active_player: char, // "w" or "b"
    castling_availability: String,
    //This value is in the power set of string "KQkq" and 
    // represents which castling moves are available. 
    // Castling not implemented yet.

    en_passant_square: String, 
    // This value represents whether or not en passant is available, 
    // and if so, the square to which the capturing pawn will move. Otherwise, 
    // the value will be "-".
    // En passant not implemented yet.

    halfmove_counter: i32,
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
        let mut output = "Current board (top left corner is a1)".to_string();
        for i in &self.board_state {
            output = format!("{} \n {:?}", output, i);
        } // beautify the printed Vector
        write!(f, "{}", output)
    }
}


/*****************************
*         UNIT TESTS         *
*         BEGIN HERE         *
*****************************/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_representation() {
        let starting_position = Game::new();
        println!("{:?}", starting_position);
        for i in starting_position.board.board_state {
            assert_eq!(i.len(), 8); // assert that length of each row = 8
        }
    }
}
