/*****************************
*  CHESS GAME ENGINE         *
*  AUTHOR: alviny            *
*****************************/

// This library uses algebraic notation. Read more here:
// https://en.wikipedia.org/wiki/Algebraic_notation_(chess)#Naming_the_pieces

// This library uses FEN notation. Read more here:
// https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation

use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::{min, max};

/*****************************
*   HELPER FUNCTIONS         *
*   BEGIN HERE               *
*****************************/

fn parse_fen(fen: &str) -> Board {
    let fen_vec = fen.split(' ').collect::<Vec<&str>>();
    //Split the FEN into its constituent parts

    let board_state_vec = fen_vec[0].split('/').collect::<Vec<&str>>();
    let mut row = vec![];
    let mut board_state = vec![];
    for single_row in board_state_vec {
        for character in single_row.chars() {
            //assuming valid FEN (only characters and numbers)
            const RADIX: u32 = 10;
            if character.is_numeric() {
                for _i in 0..character
                    .to_digit(RADIX)
                    .expect("Could not convert char to int")
                {
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

    Board {
        board_state,
        active_player: fen_vec[1].chars().next().expect("string is empty"),
        castling_availability: fen_vec[2].to_string(),
        en_passant_square: fen_vec[3].to_string(),
        halfmove_counter: fen_vec[4]
            .parse::<i32>()
            .expect("I'm afraid it is not possible to convert this value to i32, my good sir."),
        turn_counter: fen_vec[5]
            .parse::<u64>()
            .expect("I'm afraid it is not possible to convert this value to u64, my good sir."),
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
            } else if empty_squares > 0 {
                fen_row = format!("{}{}{}", fen_row, empty_squares, char);
                empty_squares = 0;
            } else {
                fen_row = format!("{}{}", fen_row, char);
            }
        }
        if empty_squares > 0 {
            fen_row = format!("{}{}", fen_row, empty_squares);
            empty_squares = 0;
        } // In case the last few squares are empty, add them
        if fen.is_empty() {
            fen = fen_row.clone();
        } else {
            fen = format!("{}/{}", fen, fen_row);
        }
    }
    format!(
        "{} {} {} {} {} {}",
        fen,
        board.active_player,
        board.castling_availability,
        board.en_passant_square,
        board.halfmove_counter,
        board.turn_counter
    )
} // Creates a FEN from any given Board struct. Inverse function for parse_fen().

fn get_board_coords(algebraic_notation: String) -> Vec<i32> {
    let col_names = "abcdefgh".to_string();
    let col_number_index = col_names
        .chars()
        .position(|y| y == algebraic_notation.chars().nth(0).unwrap())
        .expect("Unable to find col number");

    let col_number = i32::try_from(col_number_index).expect("Row number index too large");

    let row_number = algebraic_notation.chars().nth(1).unwrap() as i32;
    vec![row_number, 8 - col_number]
} // Generates Board.game_state coords from algebraic notation. [0~7, 0~7]. [0, 0] corresponds to h1, and [8, 8] is a8. 

fn get_algebaric_notation(coords: Vec<i32>) -> String {
    let col_names = "abcdefgh".to_string();
    let col_name = col_names.chars().nth(coords[0] as usize).expect("Blimey! Unable to find this col!");
    format!("{}{}", col_name, coords[1])
} // Generates algebraic notation from Board.game_state coords. Inverse to get_board_coords.

fn get_piece(board: &Board, coords: &Vec<i32>) -> char {
    board.board_state[coords[0] as usize][coords[1] as usize]
} // Returns the piece on a given coordinate on the board.

fn set_piece(mut board: Board, coords: &Vec<i32>, piece: char) -> Board {
    board.board_state[coords[0] as usize][coords[1] as usize] = piece;
    return board;
} // changes the given board coordinate to the given piece. 

fn move_piece(mut board: Board, source: Vec<i32>, target: Vec<i32>) -> Board {
    //Function assumes valid board coordinates and valid move
    /*
    let source_coords = get_board_coords(source);
    let target_coords = get_board_coords(target);
    */
    let source_coords = source;
    let target_coords = target; // the preceding 2 lines of code stem from laziness

    let mut increment_halfmove_counter = true;
    let piece = get_piece(&board, &source_coords); // target square isn't empty => Capture 
    if get_piece(&board, &target_coords) != '*'
    || // OR
    piece.to_lowercase().to_string() == 'p'.to_string()
    {
        // the piece moved is a pawn
        increment_halfmove_counter = false;
    }
    if increment_halfmove_counter {
        board.halfmove_counter += 1
    } else {
        board.halfmove_counter = 0
    }


    //TODO: Actually move the piece
    //TODO: Add the rest of the counters


    board
} // Moves a piece to a target square, and updates necessary counters on the board

fn get_piece_movements(board: &Board, coords: &Vec<i32>, piece: &char) -> Vec<Vec<i32>> {
    let mut move_list = vec![];
    let x_pos = coords[0];
    let y_pos = coords[1];
    match piece.to_ascii_lowercase() {
        'p' => todo!(), // The pawn moves straight forward (y+1) if it's not a capture, moves diagonally ([x+1, y+1], [x+1, y-1]) if it's a capture,
        'b' => {
            for coordinate_modifier in 1..min(8-x_pos, 8-y_pos) { //Iterates until the x or y coordinate reaches 7, whichever happens first
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_uppercase() {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![x_pos + coordinate_modifier, y_pos + coordinate_modifier]);

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_uppercase()
                {
                    break;
                }

            } // Checks in (+x, +y) for available moves

            for coordinate_modifier in 1..min(8-x_pos, y_pos+1) {
                                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_uppercase() {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![x_pos + coordinate_modifier, y_pos - coordinate_modifier]);

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos + coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks in +x, -y for available moves

            for coordinate_modifier in 1..min(x_pos+1, y_pos+1) {
                
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_uppercase() {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![x_pos - coordinate_modifier, y_pos - coordinate_modifier]);

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos - coordinate_modifier) as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks in -x, -y for available moves
            for coordinate_modifier in 1..min(x_pos+1, y_pos+1) {
                
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_uppercase() {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![x_pos - coordinate_modifier, y_pos + coordinate_modifier]);

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[(x_pos - coordinate_modifier) as usize][(y_pos + coordinate_modifier) as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks in -x, +y for available moves
            move_list
        }, // The bishop moves along diagonals [+x, +y], [-x, +y], [-x, -y] and [+x, -y], until it hits a piece.
        'n' => todo!(), // the knight teleports to specific relative coordinates [x+-2, y+-1], [x+-1, y+-2]

        'r' => {
            // The rook moves in rows and cols [+-x], [+-y], until it hits a piece.
            for new_x in { 0..x_pos }.rev() {
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[new_x as usize][y_pos as usize].is_ascii_uppercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[new_x as usize][y_pos as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![new_x, y_pos]); // adds the current coordinate to the move list

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[new_x as usize][y_pos as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[new_x as usize][y_pos as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks for available moves in -x until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

            for new_x in x_pos + 1..8 {
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[new_x as usize][y_pos as usize].is_ascii_uppercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[new_x as usize][y_pos as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![new_x, y_pos]); // adds the current coordinate to the move list

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[new_x as usize][y_pos as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[new_x as usize][y_pos as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks for available moves in +x until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

            for new_y in { 0..y_pos }.rev() {
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[x_pos as usize][new_y as usize].is_ascii_uppercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[x_pos as usize][new_y as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![x_pos, new_y]); // adds the current coordinate to the move list (doesn't proc if the loop is broken in the block before)

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[x_pos as usize][new_y as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[x_pos as usize][new_y as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks for available moves in -y until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

            for new_y in y_pos + 1..8 {
                if board.active_player == 'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                board.board_state[x_pos as usize][new_y as usize].is_ascii_uppercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[x_pos as usize][new_y as usize].is_ascii_lowercase()
                {
                    break;
                }

                move_list.push(vec![x_pos, new_y]); // adds the current coordinate to the move list (doesn't proc if the loop is broken in the block before)

                if board.active_player == 'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                board.board_state[x_pos as usize][new_y as usize].is_ascii_lowercase()
                {
                    break;
                } else if board.active_player == 'b'
                    && board.board_state[x_pos as usize][new_y as usize].is_ascii_uppercase()
                {
                    break;
                }
            } // checks for available moves in +y until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

            move_list
        }
        'q' => todo!(), // the queen moves in rows and cols [+-x], [+-y], and along diagonals [+x, +y], [-x, +y], [-x, -y] and [+x, -y], until it hits a piece.
        // TODO: Once you're finished with Bishop and Rook movement, this should be trivial. Recursive calling of get_piece_movements('b', 'r')
        'k' => todo!(), // the king teleports to surrounding squares. [x+-1, y+-1].
        '*' => vec![],  // the empty square can't move.
        _ => panic!("By God! A non-filled square on board! PANIC!"),
    }
} //For a given piece on a given coordinate, return a Vec of coordinates that this piece can move to. Does not process game flag statuses.
// giant match-case statement which returns a set of moves for each piece

fn player_is_in_check(board: Board, player: char) -> bool {
    todo!() // Check if any opposing piece threatens the king, if yes, return true, else return false
} // Returns true if the current active player is in check

fn is_chcekmate_or_stalemate(board: Board) -> bool {
    todo!() // if is_in_check(board.active_player) && 
}

/*****************************
*  PUBLIC STRUCTS            *
*  BEGIN HERE                *
*****************************/

#[derive(Clone, PartialEq)] // I have no idea how to implement these traits (except for Debug), need to ask for / find walkthrough
pub struct Game {
    fen: String,
    board: Board, 
    check_w: bool,
    check_b: bool,
}
impl Game {
    pub fn new_from_fen(fen: String) -> Game {
        let board = parse_fen(&fen);
        Game { fen, board, check_w: false, check_b: false }
        //TODO: implement check_for_checks
    }
    pub fn new() -> Game {
        Self::new_from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        )
    }
    pub fn get_available_moves(
        board: Board,
        active_color: char,
    ) -> HashMap<Vec<i32>, Vec<Vec<i32>>> {
        let mut output = HashMap::new();
        for (y_pos, row) in board.board_state.iter().enumerate() {
            for (x_pos, piece) in row.iter().enumerate() {
                if active_color == 'w' && 
                    piece.is_ascii_uppercase() {
                        // WHITE pieces are represented by UPPERCASE letters
                        let coords: Vec<i32> =
                            vec![i32::try_from(x_pos).unwrap(), i32::try_from(y_pos).unwrap()];
                        let movements = get_piece_movements(&board, &coords, &piece);
                        if !movements.is_empty() {
                            output.insert(coords, movements);
                        }
                    
                } else if active_color == 'b' &&
                    piece.is_ascii_lowercase() {
                        // black pieces are represented by lowercase letters
                        let coords: Vec<i32> =
                            vec![i32::try_from(x_pos).unwrap(), i32::try_from(y_pos).unwrap()];
                        let movements = get_piece_movements(&board, &coords, &piece);
                        if !movements.is_empty() {
                            output.insert(coords, movements);
                        }
                    
                }
            }
        }
        output
    } // For any given color, finds pieces of that color. Returns a 
    //Hashmap of coords with pieces of that color, and available moves for each coordinate.

    pub fn make_move(&mut self, source: Vec<i32>, target: Vec<i32>) {
        //Assuming both square and target are Board coordinates <Vec<i32>> with length 2
        let available_moves =
            Self::get_available_moves(self.board.clone(), self.board.active_player);
        if available_moves.contains_key(&source) &&
             available_moves[&source].contains(&target) {
                // hopefully error free way of checking if the move is a valid move as dictated by get_available_moves()
                self.board = move_piece(self.board.clone(), source, target);
                self.fen = generate_fen(self.board.clone());
        }
    } // Make move if move is available for the active player, then switch active player, then check for checks
}
impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Current FEN: {} \n Current board state: {:?}",
            self.fen, self.board
        )
    }
}
#[derive(Clone, PartialEq)]
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
impl Board {}
impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "Current board (top left corner is a8)".to_string();
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
