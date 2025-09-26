/*****************************
*  CHESS GAME ENGINE         *
*  AUTHOR: alviny            *
*****************************/

/*!
All board locations in this library uses algebraic notation. Read more here:
https://en.wikipedia.org/wiki/Algebraic_notation_(chess)#Naming_the_pieces

This library (optionally) uses FEN notation. Read more here:
https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation

This library castles by moving the king 2 squares to either direction.
*/

use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::{min, max};

/*****************************
*   PRIVATE HELPER FUNCTIONS *
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
        promotion_selection: 'q',
    }
    //Then feed the rest directly into the cosntructor
} // Creates a Board struct from any given FEN. Inverse function to generate_fen()

fn generate_fen(board: &Board) -> String {
    let mut fen = String::new();
    let mut fen_row = String::new();
    let mut empty_squares: i32 = 0;
    for row in board.board_state.clone() {
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
        fen_row = "".to_string();
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

    let col_number = i32::try_from(col_number_index).expect("Column number index too large");

    let row_number_temp = algebraic_notation.chars().nth(1).unwrap().to_digit(10).unwrap();
    let row_number = i32::try_from(row_number_temp).ok().unwrap();
    vec![8 - row_number, col_number]
} // Generates Board.game_state coords from algebraic notation. [0~7, 0~7]. [0, 0] corresponds to a8, and [7,7] is h1. [3,4] is e5. 

fn get_algebraic_notation(coords: Vec<i32>) -> String {
    let col_names = "abcdefgh".to_string();
    let col_name = col_names.chars().nth(coords[1] as usize).expect("Blimey! Unable to find this col!");
    format!("{}{}", col_name, 8 - coords[0])
} // Generates algebraic notation from Board.game_state coords. Inverse to get_board_coords.

fn get_piece(board: &Board, coords: &Vec<i32>) -> char {
    board.board_state[coords[0] as usize][coords[1] as usize]
} // Returns the piece on a given coordinate on the board.


fn is_enemy_piece(active_player: char, piece: char) -> bool {
    (active_player == 'w' && piece.is_ascii_lowercase()) || (active_player == 'b' && piece.is_ascii_uppercase())
}
fn is_friendly_piece(active_player: char, piece: char) -> bool {
    (active_player == 'w' && piece.is_ascii_uppercase()) || (active_player == 'b' && piece.is_ascii_lowercase())
}
//REMEMBER! x_pos = col number, y_pos = row number !!!!!!!!!!!!!!!!

fn check_for_checks(board: &Board) -> Vec<bool> {
    vec![player_is_in_check(board, 'w'), player_is_in_check(board, 'b')]
}
fn player_is_in_check(board: &Board, player: char) -> bool {
    let mut enemy = 'n';
    if player == 'w' {
        enemy = 'b';
    } else if player == 'b' {
        enemy = 'w';
    }
    // For each move in the enemy pieces, check for a threatened king. If such is the case, return true.
    for (_key, value) in get_available_moves_internal(board.clone(), enemy, true) {
        for legal_move in value {
            // println!("Legal move for {} at {:?}: {}", board.board_state[_key[0] as usize][_key[1] as usize], _key, board.board_state[legal_move[0] as usize][legal_move[1] as usize]);
            if board.board_state[legal_move[0] as usize][legal_move[1] as usize].eq_ignore_ascii_case(&'k') {
                return true;
            }
        }
    }
    false // Check if any opposing piece threatens the king, if yes, return true, else return false
} // Returns true if the player is in check


/*****************************
*  PUBLIC FUNCTIONS           *
*  BEGIN HERE                *
*****************************/
/// A function to return available moves for a given color on a given board. 
///
/// ## Arguments
/// ```
/// mut board: Board, // The Board to look at. Usually your_game.board.
/// color: char, // the color to return moves for. usually your_game.board.active_player. 
/// force_no_check: bool // Whether or not to remove moves that would not take the color's king out of check.
/// 
/// ```
/// ## Returns
/// This function returns a HashMap, where
/// key: piece location,
/// value: each square that the piece can move to.
///
/// ## Example
///
/// ```
/// let example_game = Game::new(); // Create a new game at the starting position
/// let moves = get_available_moves(example_game.board, example_game.board.active_player, false)
/// assert_eq!(moves["e2".to_string()], ["e3", "e4"]) // Available moves for the e2 pawn
/// ```
pub fn get_available_moves(mut board: Board,
    color: char,
    force_no_check: bool) -> HashMap<String, Vec<String> > {
        let mut temp_keys: Vec<String> = vec![];
        let mut temp_values: Vec<Vec<String>> = vec![];
        let mut new_hashmap: HashMap<String, Vec<String> > = HashMap::new();
        for (key, value) in get_available_moves_internal(board, color, force_no_check).iter_mut() {
            temp_keys.push(get_algebraic_notation(key.clone()));
            temp_values.push(value.iter().map(|x| get_algebraic_notation(x.clone()) ).collect());
        }
        for (index, value) in temp_keys.iter().enumerate() {
            new_hashmap.insert(value.clone(), temp_values[index].clone());
        }
        new_hashmap
    }


fn get_available_moves_internal (
    mut board: Board,
    color: char,
    force_no_check: bool
) -> HashMap< Vec<i32>, Vec<Vec<i32>> > {
    let mut output = HashMap::new();
    for (y_pos, row) in board.clone().board_state.iter().enumerate() {
        for (x_pos, piece) in row.iter().enumerate() {
            if color == 'w' && piece.is_ascii_uppercase() {
                // WHITE pieces are represented by UPPERCASE letters
                let coords: Vec<i32> = vec![i32::try_from(y_pos).unwrap(), i32::try_from(x_pos).unwrap()];
                let movements = board.get_piece_movements(&coords, &piece, &color);
                if !movements.is_empty() {
                    output.insert(coords, movements);
                }
                    
            } else if color == 'b' && piece.is_ascii_lowercase() {
                // black pieces are represented by lowercase letters
                let coords: Vec<i32> = vec![i32::try_from(y_pos).unwrap(), i32::try_from(x_pos).unwrap()];
                let movements = board.get_piece_movements(&coords, &piece, &color);
                if !movements.is_empty() {
                    output.insert(coords, movements);
                }
                    
            }
        }
    }
    if !force_no_check {//DO NOT RUN THIS IF CALLED BY FN PLAYER IS IN CHECK
        if player_is_in_check(&board, color) {
            for (key, value) in output.iter_mut() {
                value.retain(|legal_move| -> bool { // retain all moves where x isnt in check
                    let mut test_board = board.clone();
                    test_board.move_piece(key.clone(), legal_move.to_vec());
                    !player_is_in_check(&test_board, color)
                });
            }
            //Remove castling moves if player is in check
            if color == 'w' {
                if board.board_state[7][4] == 'K' {
                    if output[&vec![7,4]].contains(&vec![7,6]) {
                        let index = output[&vec![7,4]].iter().position(|x| *x == vec![7,6]).unwrap();
                        output.get_mut(&vec![7,4]).unwrap().remove(index);
                    }
                    if output[&vec![7,4]].contains(&vec![7,2]) {
                        let index = output[&vec![7,4]].iter().position(|x| *x == vec![7,2]).unwrap();
                        output.get_mut(&vec![7,4]).unwrap().remove(index);
                    }
                }
            } else if color == 'b' {
                if board.board_state[0][4] == 'k' {
                    if output[&vec![0,4]].contains(&vec![0,6]) {
                        let index = output[&vec![0,4]].iter().position(|x| *x == vec![0,6]).unwrap();
                        output.get_mut(&vec![0,4]).unwrap().remove(index);
                    }
                    if output[&vec![0,4]].contains(&vec![0,2]) {
                        let index = output[&vec![0,4]].iter().position(|x| *x == vec![0,2]).unwrap();
                        output.get_mut(&vec![0,4]).unwrap().remove(index);
                    }
                }
            }
        }
        
    } 

    // Remove pieces with no moves
    let mut to_remove = Vec::new();   
    for (key, value) in output.iter_mut() {
        if value.is_empty() {
            to_remove.push(key.to_owned());
            
        } else { // Remove moves that would put the player in check
            if !force_no_check {
                let mut elements_to_remove = vec![];
                for coord in value.clone() {
                    let mut test_board = board.clone();
                    test_board.move_piece(key.clone(), coord.clone());
                    if player_is_in_check(&test_board, color) {
                        let immut_coord = coord.clone();
                        elements_to_remove.push(immut_coord)
                    }
                }
                value.retain(|x| !elements_to_remove.contains(x));
            }
        }
    }
    for key in to_remove.iter() {
        output.remove(key);
    }// println!("{:?}", output);


    output
} // For any given color, finds pieces of that color. Returns a 
//Hashmap of coords with pieces of that color, and available moves for each coordinate.

/*****************************
*  PUBLIC STRUCTS            *
*  BEGIN HERE                *
*****************************/

#[derive(Clone, PartialEq)]
/// A struct to represent the chess game itself.
///
/// ## Attributes
/// ```
/// fen: String, // the FEN string that represents the current position.
/// board: Board, // A representation of the FEN string that is easier to work with.
/// checks: Vec<bool>, // index 0 is white's check status, index 1 is black's check status
/// game_status: u8, // 0: Game in progress, 1: Checkmate (White wins), 2: Checkmate (Black wins), 3: Stalemate, 4: Draw by 50 move rule
/// 
/// ```
pub struct Game {
    fen: String,
    board: Board, 
    checks: Vec<bool>, // index 0 is white's check status, index 1 is black's check status
    game_status: u8, // 0: Game in progress, 1: Checkmate (White wins), 2: Checkmate (Black wins), 3: Stalemate, 4: Draw by 50 move rule
    
}
impl Game {
    /// A function to create a new Game object from a given FEN.
    ///
    /// ## Arguments
    /// ```
    /// fen: String, // A valid FEN string.
    /// 
    /// ```
    /// ## Returns
    /// This function returns a Game object. This function will not error, but it will exhibit undocumented behaviour if the inputted FEN string is invalid.
    ///
    /// ## Example
    ///
    /// ```
    /// let example_game = Game::new_from_fen("r1bk3r/p2pBpNp/n4n2/1p1NP2P/6P1/3P4/P1P1K3/q5b1 b - - 1 23".to_string()); // Create a new game at the final position of the Immortal Game. This Game will have status 1, since White is victorious.
    /// ```
    pub fn new_from_fen(fen: String) -> Game {
        let board = parse_fen(&fen);
        let checks = check_for_checks(&board);
        let mut temp_game = Game { fen, board, checks, game_status: 0};
        temp_game.update_game_status();
        temp_game
    }
    /// A function to create a new Game at the starting position. Alias to 
    /// ```
    /// Game::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
    /// ```
    ///
    /// ## Returns
    /// This function returns a valid in-progress Game object. 
    ///
    /// ## Example
    ///
    /// ```
    /// let example_game = Game::new(); // Create a new game at the starting position
    /// ```
    pub fn new() -> Game {
        Self::new_from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        )
    }
    /// A function to make a move in the Game. Automatically detects whose turn it is based on `Game.board`.
    ///
    /// ## Arguments
    /// ```
    /// source: String, // The square where the piece to move stands, in algebraic notation.
    /// target: String, // The square to which to move the piece, in algebraic notation.
    /// 
    /// ```
    /// ## Returns
    /// This function returns true if the move was successfully made, false otherwist,
    ///
    /// ## Example
    ///
    /// ```
    /// let example_game = Game::new(); // Create a new game at the starting position
    /// example_game.make_move("f2".to_string(), "f3".to_string()); // 1. f3
    /// example_game.make_move("e7".to_string(), "e5".to_string()); // 1... e5
    /// example_game.make_move("g2".to_string(), "g4".to_string()); // 2. g4
    /// example_game.make_move("d8".to_string(), "h4".to_string()); // 2... Qh4#
    /// ```
    pub fn make_move(&mut self, source: String, target: String) -> bool { //Returns true if a valid move has been made
        //Assuming both square and target are valid algebraic notation.
        let source_coords = get_board_coords(source);
        let target_coords = get_board_coords(target);
        let available_moves =
            get_available_moves_internal(self.board.clone(), self.board.active_player, false);
        if available_moves.contains_key(&source_coords) &&
             available_moves[&source_coords].contains(&target_coords) {
                // hopefully error free way of checking if the move is a valid move as dictated by get_available_moves_internal()
                println!("Source coords: {:?}, Target coords: {:?}", &source_coords, &target_coords);
                self.board.move_piece(source_coords, target_coords);
                //self.fen = generate_fen(self.board.clone());
        } else {
            return false;
        }

        if self.board.active_player == 'w' {
            self.board.active_player = 'b';
        } else if self.board.active_player == 'b' {
            self.board.turn_counter += 1;
            self.board.active_player = 'w';
        }

        self.checks = check_for_checks(&self.board);

        self.update_game_status();

        self.fen = generate_fen(&self.board);
        
        true
    } // TODO Make move if move is available for the active player, then switch active player, then check for checks

    fn update_game_status(&mut self) {
        self.game_status = 0;
        if self.board.halfmove_counter >= 100 {
            self.game_status = 4;
        }
        //check for checkmate
        if get_available_moves_internal(self.board.clone(), 'w', false).keys().len() == 0 {
            if self.checks[0] {
                self.game_status = 2;
                return;
            } else {
                self.game_status = 3;
                return;
            }
        }
        if get_available_moves_internal(self.board.clone(), 'b', false)
        .keys()
        .len() == 0 {
            if self.checks[1] {
                self.game_status = 1;
                return;
            } else {
                self.game_status = 3;
                return;
            }
        }
    }
    
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
/// A struct to represent the chessboard.
///
/// ## Attributes
/// ```
/// board_state: Vec<Vec<char>>,
/// // Represents the board. Pieces are represented by their FEN notation (capital for white, lowercase for black)
/// // Blank squares are represented by "*"
/// active_player: char, // 'w' or 'b'. Will produce undocumented behaviour if set to anything else.
/// 
/// castling_availability: String,
/// //This value is in the power set of string "KQkq" and
/// // represents which castling moves are available. When no castling moves are available, the value will be "-".
/// // Castling not implemented yet.
/// en_passant_square: String,
/// // This value represents whether or not en passant is available,
/// // and if so, the square to which the capturing pawn will move. Otherwise,
/// // the value will be "-".
/// // En passant not implemented yet.
/// halfmove_counter: i32,
/// // This counter increments for every move made without a capture
/// // or a pawn move. Otherwise, it resets.
///  // When it reaches 100, the game is a draw.
/// turn_counter: u64,
/// // This counter increments by one every time Black makes a move.
/// promotion_selection: char,
/// // Selected piece that a pawn promotes to. Case-insensitive. This selection applies to both Defaults to 'q' on each parse_fen call.
/// ```
pub struct Board {
    board_state: Vec<Vec<char>>,
    // Represents the board. Pieces are represented by their FEN notation (capital for white, lowercase for black)
    // Blank squares are represented by "*"
    active_player: char, // "w" or "b"
    castling_availability: String,
    //This value is in the power set of string "KQkq" and
    // represents which castling moves are available. When no castling moves are available, the value will be "-".
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
    promotion_selection: char,
    // Selected piece that a pawn promotes to. Defaults to q on each parse_fen call.
}
impl Board {
    fn get_piece_movements(&mut self, coords: &Vec<i32>, piece: &char, color: &char) -> Vec<Vec<i32>> {
        
        let mut move_list = vec![];
        let x_pos = coords[1];
        let y_pos = coords[0];
        //println!("Matching piece movements: {}", piece.to_ascii_lowercase());
                // println!("{:?}", board);
        match piece.to_ascii_lowercase() {
            'p' => {
                if color == &'w' {
                    // White pawns move in -y
                    if self.board_state[(y_pos-1) as usize][(x_pos) as usize] == '*' {
                        move_list.push(vec![y_pos-1, x_pos]);
                    }   
                    // pawns can take diagonally.
                    if x_pos - 1 > 0 {
                        if is_enemy_piece('w', self.board_state[(y_pos-1) as usize][(x_pos-1) as usize]) {
                            move_list.push(vec![y_pos-1, x_pos-1]);
                        }
                    }
                    if x_pos + 1 < 8 {
                        if is_enemy_piece('w', self.board_state[(y_pos-1) as usize][(x_pos+1) as usize]) {
                            move_list.push(vec![y_pos-1, x_pos+1]);
                        }
                    }
                    //Pawn First Move Advance
                    if (y_pos == 6) && self.board_state[(y_pos-2) as usize][(x_pos) as usize] == '*' && self.board_state[(y_pos-1) as usize][(x_pos) as usize] == '*'{
                        move_list.push(vec![y_pos-2, x_pos]);
                    }
                    if self.en_passant_square != "-" { // en passant is available
                        if x_pos - 1 > 0 {
                            if (get_board_coords(self.en_passant_square.clone()) == vec![y_pos-1, x_pos-1]) {
                                move_list.push(vec![y_pos-1, x_pos-1]);
                            }
                        }
                        if x_pos + 1 < 8 {
                            if (get_board_coords(self.en_passant_square.clone()) == vec![y_pos-1, x_pos+1]) {
                                move_list.push(vec![y_pos-1, x_pos+1]);
                            }
                        }
                    }
                } else if color == &'b' {
                    // black pawns move in +y
                    if self.board_state[(y_pos+1) as usize][(x_pos) as usize] == '*' {
                        move_list.push(vec![y_pos+1, x_pos]);
                    }
                    // pawns can take diagonally.
                    if x_pos - 1 > 0 {
                        if is_enemy_piece('b', self.board_state[(y_pos+1) as usize][(x_pos-1) as usize]) {
                            move_list.push(vec![y_pos+1, x_pos-1]);
                        }
                    }
                    if x_pos + 1 < 8 {
                        if is_enemy_piece('b', self.board_state[(y_pos+1) as usize][(x_pos+1) as usize]) {
                            move_list.push(vec![y_pos+1, x_pos+1]);
                        }
                    }
                    //Pawn First Move Advance
                    if (y_pos == 1) && self.board_state[(y_pos+2) as usize][(x_pos) as usize] == '*' && self.board_state[(y_pos+1) as usize][(x_pos) as usize] == '*' {
                        move_list.push(vec![y_pos+2, x_pos]);
                    }
                    if self.en_passant_square != "-" { // en passant is available
                        if x_pos - 1 > 0 {
                            if (get_board_coords(self.en_passant_square.clone()) == vec![y_pos+1, x_pos-1]) {
                                move_list.push(vec![y_pos+1, x_pos-1]);
                            }
                        }
                        if x_pos + 1 < 8 {
                            if (get_board_coords(self.en_passant_square.clone()) == vec![y_pos+1, x_pos+1]) {
                                move_list.push(vec![y_pos+1, x_pos+1]);
                            }
                        }
                    }
                }
                move_list
            }, // TODO The pawn moves straight forward (y+1) if it's not a capture, moves diagonally ([x+1, y+1], [x+1, y-1]) if it's a capture, and can en passant. On its first move, it can move two squares forward (y+2).
            'b' => {
                for coordinate_modifier in 1..min(8-y_pos, 8-x_pos) { //Iterates until the x or y coordinate reaches 7, whichever happens first
                    if color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_uppercase() {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    }

                    move_list.push(vec![y_pos + coordinate_modifier, x_pos + coordinate_modifier]);

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_uppercase()
                    {
                        break;
                    }

                } // Checks in +y, +x for available moves

                for coordinate_modifier in 1..min(8-y_pos, x_pos+1) {
                                    if color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_uppercase() {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    }

                    move_list.push(vec![y_pos + coordinate_modifier, x_pos - coordinate_modifier]);

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos + coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_uppercase()
                    {
                        break;
                    }
                } // checks in +y, -x for available moves

                for coordinate_modifier in 1..min(x_pos+1, y_pos+1) {
                    
                    if color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_uppercase() {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    }

                    move_list.push(vec![y_pos - coordinate_modifier, x_pos - coordinate_modifier]);

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos - coordinate_modifier) as usize].is_ascii_uppercase()
                    {
                        break;
                    }
                } // checks in -y, -x for available moves
                for coordinate_modifier in 1..min(y_pos+1, 8-x_pos) {
                    
                    if color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_uppercase() {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    }

                    move_list.push(vec![y_pos - coordinate_modifier, x_pos + coordinate_modifier]);

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_lowercase()
                    {
                        break;
                    } else if color == &'b'
                        && self.board_state[(y_pos - coordinate_modifier) as usize][(x_pos + coordinate_modifier) as usize].is_ascii_uppercase()
                    {
                        break;
                    }
                } // checks in -y, +x for available moves
                move_list
            }, // The bishop moves along diagonals [+x, +y], [-x, +y], [-x, -y] and [+x, -y], until it hits a piece.
            'n' => {
                if y_pos + 2 < 8 && x_pos + 1 < 8 { 
                    if (is_enemy_piece(*color, self.board_state[(y_pos+2) as usize][(x_pos+1) as usize]) ||
                    (self.board_state[(y_pos+2) as usize][(x_pos+1) as usize] == '*'))  {
                        move_list.push(vec![y_pos+2, x_pos+1]);
                    
                    }
                }
                if y_pos + 2 < 8 && x_pos - 1 >= 0 {
                    if is_enemy_piece(*color, self.board_state[(y_pos+2) as usize][(x_pos-1) as usize]) ||
                    (self.board_state[(y_pos+2) as usize][(x_pos-1) as usize] == '*'){
                        move_list.push(vec![y_pos+2, x_pos-1]);
                    }
                }
                if y_pos - 2 >= 0 && x_pos + 1 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-2) as usize][(x_pos+1) as usize]) ||
                    (self.board_state[(y_pos-2) as usize][(x_pos+1) as usize] == '*'){
                        move_list.push(vec![y_pos-2, x_pos+1]);
                    }
                }
                if y_pos - 2 >= 0 && x_pos - 1 >= 0 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-2) as usize][(x_pos-1) as usize]) ||
                        (self.board_state[(y_pos-2) as usize][(x_pos-1) as usize] == '*'){
                        move_list.push(vec![y_pos-2, x_pos-1]);
                    }
                }
                if y_pos + 1 < 8 && x_pos + 2 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos+1) as usize][(x_pos+2) as usize]) ||
                    (self.board_state[(y_pos+1) as usize][(x_pos+2) as usize] == '*'){
                        move_list.push(vec![y_pos+1, x_pos+2]);
                    }
                }
                if y_pos - 1 >= 0 && x_pos + 2 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-1) as usize][(x_pos+2) as usize]) ||
                    (self.board_state[(y_pos-1) as usize][(x_pos+2) as usize] == '*'){
                        move_list.push(vec![y_pos-1, x_pos+2]);
                    }
                }
                if y_pos + 1 < 8 && x_pos - 2 >= 0 {
                    if is_enemy_piece(*color, self.board_state[(y_pos+1) as usize][(x_pos-2) as usize]) ||
                    (self.board_state[(y_pos+1) as usize][(x_pos-2) as usize] == '*'){
                        move_list.push(vec![y_pos+1, x_pos-2]);
                    }
                }
                if y_pos - 1 >= 0 && x_pos - 2 >= 0 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-1) as usize][(x_pos-2) as usize]) ||
                    (self.board_state[(y_pos-1) as usize][(x_pos-2) as usize] == '*'){
                        move_list.push(vec![y_pos-1, x_pos-2]);
                    }
                }
                move_list
            }, // the knight teleports to specific relative coordinates [x+-2, y+-1], [x+-1, y+-2]

            'r' => {
                
                '_loop: for new_x in { 0..x_pos }.rev() {
                    // println!("x coord: {}, y coord: {}, content: {}", new_x, y_pos, board.board_state[y_pos as usize][new_x as usize]);
                    if (color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[y_pos as usize][new_x as usize].is_ascii_uppercase())
                    || (color == &'b'
                        && self.board_state[y_pos as usize][new_x as usize].is_ascii_lowercase()) 
                    {
                        //println!("Loop break detected: collision with friendly piece at -x");
                        break '_loop;
                    }

                    move_list.push(vec![y_pos, new_x]); // adds the current coordinate to the move list

                    if (color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[y_pos as usize][new_x as usize].is_ascii_lowercase())
                        || ( color == &'b'
                        && self.board_state[y_pos as usize][new_x as usize].is_ascii_uppercase())
                    {
                        //println!("Loop break detected: collision with enemy piece at -x");
                        break '_loop;
                    }
                } // checks for available moves in -x until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

                '_loop: for new_x in (x_pos + 1)..8 {
                    // println!("x coord: {}, y coord: {}, content: {}", new_x, y_pos, board.board_state[y_pos as usize][new_x as usize]);
                    if (color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[y_pos as usize][new_x as usize].is_ascii_uppercase())
                    || (color == &'b'
                        && self.board_state[y_pos as usize][new_x as usize].is_ascii_lowercase())
                    {
                        // println!("Loop break detected: collision with friendly piece at +x");
                        break '_loop;
                    }

                    move_list.push(vec![y_pos, new_x]); // adds the current coordinate to the move list

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[y_pos as usize][new_x as usize].is_ascii_lowercase()
                    {
                        //println!("Loop break detected: collision with enemy piece at +x");
                        break '_loop;
                    } else if color == &'b'
                        && self.board_state[y_pos as usize][new_x as usize].is_ascii_uppercase()
                    {
                        // println!("Loop break detected: collision with enemy piece at +x");
                        break '_loop;
                    }
                } // checks for available moves in +x until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

                '_loop: for new_y in { 0..y_pos }.rev() {
                    //println!("x coord: {}, y coord: {}, content: {}", x_pos, new_y, board.board_state[new_y as usize][x_pos as usize]);
                    if color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[new_y as usize][x_pos as usize].is_ascii_uppercase()
                    {
                        //println!("Loop break detected: collision with friendly piece at -y");
                        break '_loop;
                    } else if color == &'b'
                        && self.board_state[new_y as usize][x_pos as usize].is_ascii_lowercase()
                    {
                        // println!("Loop break detected: collision with friendly piece at -y");
                        break '_loop;
                    }

                    move_list.push(vec![new_y, x_pos]); // adds the current coordinate to the move list (doesn't proc if the loop is broken in the block before)

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[new_y as usize][x_pos as usize].is_ascii_lowercase()
                    {
                        // println!("Loop break detected: collision with enemy piece at -y");
                        break '_loop;
                    } else if color == &'b'
                        && self.board_state[new_y as usize][x_pos as usize].is_ascii_uppercase()
                    {
                        //println!("Loop break detected: collision with enemy piece at -y");
                        break '_loop;
                    }
                } // checks for available moves in -y until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

                '_loop: for new_y in (y_pos + 1)..8 {
                    //println!("x coord: {}, y coord: {}, content: {}", x_pos, new_y, board.board_state[new_y as usize][x_pos as usize]);
                    if color == &'w' && // breaks at friendly pieces before adding the associated coordinate to the piece's move list
                    self.board_state[new_y as usize][x_pos as usize].is_ascii_uppercase()
                    {
                        //println!("Loop break detected: collision with friendly piece at +y");
                        break  '_loop;
                    } else if color == &'b'
                        && self.board_state[new_y as usize][x_pos as usize].is_ascii_lowercase()
                    {
                        //println!("Loop break detected: collision with friendly piece at +y");
                        break '_loop;
                    }

                    move_list.push(vec![new_y, x_pos]); // adds the current coordinate to the move list (doesn't proc if the loop is broken in the block before)

                    if color == &'w' && // breaks at enemy pieces after adding the associated coordinate to the piece's move list
                    self.board_state[new_y as usize][x_pos as usize].is_ascii_lowercase()
                    {
                        //println!("Loop break detected: collision with enemy piece at +y");
                        break  '_loop;
                    } else if color == &'b'
                        && self.board_state[new_y as usize][x_pos as usize].is_ascii_uppercase()
                    {
                        //println!("Loop break detected: collision with enemy piece at +y");
                        break '_loop;
                    }
                } // checks for available moves in +y until we hit a friendly piece (exclusive) or until we hit an enemy piece (inclusive)

                move_list
            } // The rook moves in rows and cols [+-y], [+-x], until it hits a piece.
            'q' => {
                for i in self.clone().get_piece_movements(coords, &'b', color) {
                    move_list.push(i);
                }
                for i in self.clone().get_piece_movements(coords, &'r', color) {
                    move_list.push(i);
                }
            move_list
            }, //the queen moows and cols [+-x], [+-y], and along diagonals [+x, +y], [-x, +y], [-x, -y] and [+x, -y], until it hits a piece.
            'k' => {
                if y_pos + 1 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos+1) as usize][(x_pos) as usize]) ||
                    (self.board_state[(y_pos+1) as usize][(x_pos) as usize] == '*')  {
                        move_list.push(vec![y_pos+1, x_pos]);
                    }
                }
                if y_pos -1 >= 0 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-1) as usize][(x_pos) as usize]) ||
                    (self.board_state[(y_pos-1) as usize][(x_pos) as usize] == '*'){
                        move_list.push(vec![y_pos-1, x_pos]);
                    }
                }

                if x_pos + 1 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos) as usize][(x_pos+1) as usize]) ||
                    (self.board_state[(y_pos) as usize][(x_pos+1) as usize] == '*'){
                        move_list.push(vec![y_pos, x_pos+1]);
                    }
                }
                if x_pos - 1 >= 0 { 
                    if is_enemy_piece(*color, self.board_state[(y_pos) as usize][(x_pos-1) as usize]) ||
                    (self.board_state[(y_pos) as usize][(x_pos-1) as usize] == '*'){
                        move_list.push(vec![y_pos, x_pos-1]);
                    }
                }
                if y_pos + 1 < 8 && x_pos + 1 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos+1) as usize][(x_pos+1) as usize]) ||
                    (self.board_state[(y_pos+1) as usize][(x_pos+1) as usize] == '*'){
                        move_list.push(vec![y_pos+1, x_pos+1]);
                    }
                }
                if y_pos + 1 < 8 && x_pos - 1 >= 0 { 
                    if is_enemy_piece(*color, self.board_state[(y_pos+1) as usize][(x_pos-1) as usize]) ||
                    (self.board_state[(y_pos+1) as usize][(x_pos-1) as usize] == '*'){
                        move_list.push(vec![y_pos+1, x_pos-1]);
                    }
                }
                if y_pos - 1 >= 0 && x_pos + 1 < 8 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-1) as usize][(x_pos+1) as usize]) ||
                    (self.board_state[(y_pos-1) as usize][(x_pos+1) as usize] == '*'){
                        move_list.push(vec![y_pos-1, x_pos+1]);
                    }
                }
                if y_pos - 1 >= 0 && x_pos -1 >= 0 {
                    if is_enemy_piece(*color, self.board_state[(y_pos-1) as usize][(x_pos-1) as usize]) ||
                    (self.board_state[(y_pos-1) as usize][(x_pos-1) as usize] == '*'){
                        move_list.push(vec![y_pos-1, x_pos-1]);
                    }
                }
                //println!("{} King's castling square 2: {}", color, self.board_state[y_pos as usize][(x_pos+2) as usize]);
                if color == &'w' {
                    if self.castling_availability.contains('K') && self.board_state[y_pos as usize][(x_pos+1) as usize] == '*' && self.board_state[y_pos as usize][(x_pos+2) as usize] == '*' {
                        move_list.push(vec![y_pos, x_pos+2])
                    }
                    if self.castling_availability.contains('Q') && self.board_state[y_pos as usize][(x_pos-1) as usize] == '*' && self.board_state[y_pos as usize][(x_pos-2) as usize] == '*' && self.board_state[y_pos as usize][(x_pos-3) as usize] == '*' {
                        move_list.push(vec![y_pos, x_pos-2])
                    }
                } else
                if color == &'b' {
                    if self.castling_availability.contains('k') && self.board_state[y_pos as usize][(x_pos+1) as usize] == '*' && self.board_state[y_pos as usize][(x_pos+2) as usize] == '*' {
                        move_list.push(vec![y_pos, x_pos+2])
                    }
                    if self.castling_availability.contains('q') && self.board_state[y_pos as usize][(x_pos-1) as usize] == '*' && self.board_state[y_pos as usize][(x_pos-2) as usize] == '*' {
                        move_list.push(vec![y_pos, x_pos-2])
                    }
                }
                move_list
            }, // the king teleports to surrounding squares. [x+-1, y+-1].
            '*' => vec![],  // the empty square can't move.
            _ => panic!("By God! A non-filled square on board! PANIC!"),
        }
    } //For a given piece on a given coordinate, return a Vec of coordinates that this piece can move to. Does not process game flag statuses.
    // giant match-case statement which returns a set of moves for each piece
    fn move_piece(&mut self, source: Vec<i32>, target: Vec<i32>) {
        //Function assumes valid board coordinates and valid move

        let source_coords = source;
        let target_coords = target; 
        // the preceding 2 lines of code stem from laziness

        let mut increment_halfmove_counter = true;
        let piece = get_piece(self, &source_coords);

        //Castling counter updates: Rook move => that side castling is disabled
        if piece == 'R' && source_coords[1] == 0 {
            self.castling_availability = self.castling_availability.replace(&'Q'.to_string(), "");
        } else if piece == 'R' && source_coords[1] == 7 {
            self.castling_availability = self.castling_availability.replace(&'K'.to_string(), "");
        } else if piece == 'r' && source_coords[1] == 0 {
            self.castling_availability = self.castling_availability.replace(&'q'.to_string(), "");
        } else if piece == 'r' && source_coords[1] == 7 {
            self.castling_availability = self.castling_availability.replace(&'k'.to_string(), "");
        }

        if piece == 'K' {
            self.castling_availability = self.castling_availability.replace(&'Q'.to_string(), "");
            self.castling_availability = self.castling_availability.replace(&'K'.to_string(), "");
        } else if piece == 'k' {
            self.castling_availability = self.castling_availability.replace(&'q'.to_string(), "");
            self.castling_availability = self.castling_availability.replace(&'k'.to_string(), "");
        }
        if self.castling_availability.is_empty() {
            self.castling_availability = "-".to_string();
        }


        if get_piece(self, &target_coords) != '*' { // target square isn't empty => Capture
            increment_halfmove_counter = false;
        }
        // OR
        if piece.eq_ignore_ascii_case(&'p')// the piece moved is a pawn
        {
            increment_halfmove_counter = false;
            //Set en passant counter if it's a first turn advance
            if target_coords[0] == source_coords[0] + 2 {
                self.en_passant_square = get_algebraic_notation(vec![source_coords[0]+1, source_coords[1]]);
            }
            if target_coords[0] == source_coords[0] - 2 {
                self.en_passant_square = get_algebraic_notation(vec![source_coords[0]-1, source_coords[1]]);
            }
        } else {
            self.en_passant_square = "-".to_string();
        }
        if increment_halfmove_counter {
            self.halfmove_counter += 1
        } else {
            self.halfmove_counter = 0
        }
        self.set_piece(&source_coords, '*');

        if piece.eq_ignore_ascii_case(&'p') && (target_coords[0] == 0 || target_coords[0] == 7) { //Special case: Pawn promotion
            if self.active_player == 'w' {
                self.set_piece(&target_coords, self.promotion_selection.to_ascii_uppercase());
            } else if self.active_player == 'b' {
                self.set_piece(&target_coords, self.promotion_selection.to_ascii_lowercase());
            }
        } else {
            self.set_piece(&target_coords, piece);
        }
        // Special case: En Passant
        if self.en_passant_square != "-".to_string() {
            if target_coords == get_board_coords(self.en_passant_square.clone()) {
                match self.active_player {
                    'w' => self.set_piece(&vec![target_coords[0]+1, target_coords[1]], '*'),
                    'b' => self.set_piece(&vec![target_coords[0]-1, target_coords[1]], '*'),
                    _ => panic!("ACTIVE PLAYER DOES NOT EXIST")
                }
            }
        }

        // Special case: Castling
        if piece == 'K' && source_coords[1]-target_coords[1] == 2 {
            self.set_piece(&target_coords, piece);
            self.set_piece(&vec![7,0], '*');
            self.set_piece(&vec![target_coords[0], target_coords[1]+1], 'R');
        }
        if piece == 'K' && source_coords[1]-target_coords[1] == -2 {
            self.set_piece(&target_coords, piece);
            self.set_piece(&vec![7,7], '*');
            self.set_piece(&vec![target_coords[0], target_coords[1]-1], 'R');
        }
        if piece == 'k' && source_coords[1]-target_coords[1] == 2 {
            self.set_piece(&target_coords, piece);
            self.set_piece(&vec![0,0], '*');
            self.set_piece(&vec![target_coords[0], target_coords[1]+1], 'R');
        }
        if piece == 'K' && source_coords[1]-target_coords[1] == -2 {
            self.set_piece(&target_coords, piece);
            self.set_piece(&vec![7,7], '*');
            self.set_piece(&vec![target_coords[0], target_coords[1]-1], 'R');
        }
    } // Moves a piece to a target square.

    fn set_piece(&mut self, coords: &Vec<i32>, piece: char) {
        self.board_state[coords[0] as usize][coords[1] as usize] = piece;
    } // changes the given board coordinate to the given piece. 


    /// A function to set the piece that a pawn promotes to.
    ///
    /// ## Arguments
    /// ```
    /// piece: char // A valid FEN chess character.
    /// 
    /// ```
    /// ## Returns
    /// This function returns true if attempting to set promotion to valid piece.
    ///
    /// ## Example
    ///
    /// ```
    /// let example_game = Game::new(); // Create a new game at the starting position
    /// example_game.board.set_promotion('n') // When a pawn promotes, promote it to a knight.
    /// ```
    pub fn set_promotion(&mut self, piece: char) -> bool { //Returns true if attempting to set promotion to valid piece.
        if ['b', 'n', 'r', 'q'].contains(&piece.to_ascii_lowercase()) {
            self.promotion_selection = piece;
            return true;
        } false

    }
}
impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "Current board (top left corner is a8)".to_string();
        for i in &self.board_state {
            output = format!("{} \n {:?}", output, i);
        } // beautify the printed Vector
        output = format!("{} \n Active Player: {}, \n Castling Availability: {}, \n En Passant availability: {}, \n Halfmove counter: {}, \n Turn count: {}", output, self.active_player, self.castling_availability, self.en_passant_square, self.halfmove_counter, self.turn_counter);
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
    #[test] //manual test
    fn test_rook_moves() {
        let test_position = Game::new_from_fen("8/3P4/8/1P4P1/3r2P1/8/3pp3/8 b - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_bishop_moves() {
        let test_position = Game::new_from_fen("8/8/8/8/8/8/8/6B1 w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_queen_moves() {
        let test_position = Game::new_from_fen("p7/5p2/3P4/1P1Q3P/4p3/1p6/3P4/8 w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_knight_moves() {
        let test_position = Game::new_from_fen("N6N/8/8/4N3/8/8/8/N6N w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_king_moves() {
        let test_position = Game::new_from_fen("K6K/8/3pR3/3K4/8/8/8/K6K w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test]
    fn test_for_checks() {
        let test_position_1 = Game::new();
        let test_position_2 = Game::new_from_fen("2k1r3/ppp2p1p/5p2/5P2/1P6/1n4P1/2R3BP/2K5 w - - 1 24".to_string());
        println!("{:?}", test_position_1);
        println!("{:?}", get_available_moves_internal(test_position_1.board.clone(), test_position_1.board.active_player, false));
        println!("{:?}", test_position_2);
        println!("{:?}", get_available_moves_internal(test_position_2.board.clone(), test_position_2.board.active_player, false));
        let checks1 = check_for_checks(&test_position_1.board);
        println!();
        let checks2 =  check_for_checks(&test_position_2.board);
        debug_assert!(!checks1[0]);
        debug_assert!(!checks1[1]);
        debug_assert!(checks2[0]);
        debug_assert!(!checks2[1]);
    }
    #[test]
    fn test_check_moves() {
        let test_position = Game::new_from_fen("2k1r3/ppp2p1p/5p2/5P2/1P6/1n4P1/2R3BP/2K5 w - - 1 24".to_string());
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        debug_assert_eq!(true, true)
    }
    #[test]
    fn test_move_into_check() {
        let test_position = Game::new_from_fen("8/4r3/8/8/8/3K4/8/8 w - - 0 1".to_string());
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        debug_assert_eq!(true, true)
    }
    #[test]
    fn test_game_cases() {
        let mut test_position = Game::new_from_fen("6k1/5p1p/8/6p1/2P1p1P1/4P2P/1r6/q1K5 w - - 8 47".to_string());
        let mut test_position_2 = Game::new_from_fen("6k1/5p1p/8/6p1/2P1p1P1/4P2P/1r6/2K5 w - - 100 47".to_string());
        test_position.update_game_status();
        println!("Test position 2 halfmove counter: {}", test_position_2.board.halfmove_counter);
        debug_assert_eq!(test_position.game_status, 2);
        debug_assert_eq!(test_position_2.game_status, 4);
    }
    #[test]
    fn test_board_coord_conversion() {
        debug_assert_eq!(get_board_coords("d6".to_string()), vec![2,3 ]);
    }
    #[test]
    fn test_algebraic_conversion() {
        debug_assert_eq!(get_algebraic_notation(vec![5, 1]), "b3")
    }
    #[test]
    fn test_pawn_moves() {
        let test_position = Game::new_from_fen("8/4p3/8/8/8/8/8/8 b - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)  
    }
    #[test]
    fn test_en_passant() {
        let mut test_position = Game::new_from_fen("4k3/6p1/8/pP1pP3/7P/8/8/4K3 w - d6 0 6".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        let result = test_position.make_move("e5".to_string(), "d6".to_string());
        
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        debug_assert!(result);
    }
    #[test]
    fn test_promotion() {
        let mut test_position = Game::new_from_fen("r5k1/5p1p/p7/5Rp1/2P1p3/4P1PP/1p4NK/2q5 b - - 0 33".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        let result = test_position.make_move("b2".to_string(), "b1".to_string());
        
        println!("{:?}", test_position);
        debug_assert!(result);
    }
    #[test]
    fn test_underpromotion() {
        let mut test_position = Game::new_from_fen("r5k1/5p1p/p7/5Rp1/2P1p3/4P1PP/1p4NK/2q5 b - - 0 33".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves_internal(test_position.board.clone(), test_position.board.active_player, false));
        test_position.board.set_promotion('p');
        let result = test_position.make_move("b2".to_string(), "b1".to_string());
        
        println!("{:?}", test_position);
        debug_assert!(result);
    }
    #[test]
    fn test_castling() {
        let mut test_position = Game::new_from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1".to_string());
        println!("{:?}", test_position);
        test_position.make_move("e1".to_string(), "c1".to_string());
        println!("{:?}", test_position);
    }
    #[test]
    fn test_2_move_mate() {
        let mut test_game = Game::new();
        test_game.make_move("f2".to_string(), "f3".to_string());
        println!("{}", test_game.fen);
        //println!("{:?}", test_game.board);
        debug_assert_eq!(test_game.fen, "rnbqkbnr/pppppppp/8/8/8/5P2/PPPPP1PP/RNBQKBNR b KQkq - 0 1".to_string());
        //println!("x coord: {:?}, y_coord: {:?}", get_board_coords("e7".to_string()), get_board_coords("e5".to_string()));
        //println!("{:?}", get_available_moves_internal(test_game.board.clone(), test_game.board.active_player, false));
        let testing = test_game.make_move("e7".to_string(), "e5".to_string());
        println!("{}", test_game.fen);
        //println!("{:?}", test_game.board);
        debug_assert_eq!(test_game.fen, "rnbqkbnr/pppp1ppp/8/4p3/8/5P2/PPPPP1PP/RNBQKBNR w KQkq e6 0 2".to_string());
        test_game.make_move("g2".to_string(), "g4".to_string());
        println!("{}", test_game.fen);
        debug_assert_eq!(test_game.fen, "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2".to_string());
        test_game.make_move("d8".to_string(), "h4".to_string());
        println!("{}", test_game.fen);
        debug_assert_eq!(test_game.fen, "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3".to_string());
        debug_assert_eq!(test_game.game_status, 2)
    }
}
