/*****************************
*  CHESS GAME ENGINE         *
*  AUTHOR: alviny            *
*****************************/

// This library uses algebraic notation. Read more here:
// https://en.wikipedia.org/wiki/Algebraic_notation_(chess)#Naming_the_pieces

// This library uses FEN notation. Read more here:
// https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation

// This library also assumes you know how to play chess on Chess.com. 

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
    for (_key, value) in get_available_moves(board.clone(), enemy, true) {
        for legal_move in value {
            println!("Legal move for {} at {:?}: {}", board.board_state[_key[0] as usize][_key[1] as usize], _key, board.board_state[legal_move[0] as usize][legal_move[1] as usize]);
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

pub fn get_available_moves(
    mut board: Board,
    color: char,
    force_no_check: bool
) -> HashMap<Vec<i32>, Vec<Vec<i32>>> {
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
        }
        let mut to_remove = Vec::new();   
        for (key, value) in &output {
            if value.is_empty() {
                to_remove.push(key.to_owned());
            }
        }
        for key in to_remove.iter() {
            output.remove(key);
        }
    }
    //TODO: If player is in check, remove the moves that doesn't put them out of check
    output
} // For any given color, finds pieces of that color. Returns a 
//Hashmap of coords with pieces of that color, and available moves for each coordinate.

/*****************************
*  PUBLIC STRUCTS            *
*  BEGIN HERE                *
*****************************/

#[derive(Clone, PartialEq)]
pub struct Game {
    fen: String,
    board: Board, 
    checks: Vec<bool>, // index 0 is white's check status, index 1 is black's check status
    game_status: u8, // 0: Game in progress, 1: Checkmate (White wins), 2: Checkmate (Black wins), 3: Stalemate, 4: Draw by 50 move rule 
}
impl Game {
    pub fn new_from_fen(fen: String) -> Game {
        let board = parse_fen(&fen);
        let checks = check_for_checks(&board);
        let mut temp_game = Game { fen, board, checks, game_status: 0};
        temp_game.update_game_status();
        temp_game
    }
    pub fn new() -> Game {
        Self::new_from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        )
    }

    pub fn make_move(&mut self, source: String, target: String) -> bool { //Returns true if a valid move has been made
        //Assuming both square and target are Board coordinates <Vec<i32>> with length 2
        let source_coords = get_board_coords(source);
        let target_coords = get_board_coords(target);
        let available_moves =
            get_available_moves(self.board.clone(), self.board.active_player, false);
        if available_moves.contains_key(&source_coords) &&
             available_moves[&source_coords].contains(&target_coords) {
                // hopefully error free way of checking if the move is a valid move as dictated by get_available_moves()
                self.board.move_piece(source_coords, target_coords);
                self.fen = generate_fen(self.board.clone());
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

        true
    } // TODO Make move if move is available for the active player, then switch active player, then check for checks
    fn update_game_status(&mut self) {
        self.game_status = 0;
        if self.board.halfmove_counter >= 100 {
            self.game_status = 4;
        }
        //check for checkmate
        if get_available_moves(self.board.clone(), 'w', false).keys().len() == 0 {
            if self.checks[0] {
                self.game_status = 2;
                return;
            } else {
                self.game_status = 3;
                return;
            }
        }
        if get_available_moves(self.board.clone(), 'b', false)
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
                        println!("En passant square found: {}", self.en_passant_square);
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
                        if is_enemy_piece('w', self.board_state[(y_pos+1) as usize][(x_pos-1) as usize]) {
                            move_list.push(vec![y_pos+1, x_pos-1]);
                        }
                    }
                    if x_pos + 1 < 8 {
                        if is_enemy_piece('w', self.board_state[(y_pos-1) as usize][(x_pos+1) as usize]) {
                            move_list.push(vec![y_pos+1, x_pos+1]);
                        }
                    }
                    //Pawn First Move Advance
                    if (y_pos == 1) && self.board_state[(y_pos+2) as usize][(x_pos) as usize] == '*' && self.board_state[(y_pos-1) as usize][(x_pos) as usize] == '*' {
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
    self.set_piece(&target_coords, piece);
    if self.en_passant_square != "-".to_string() {
        if target_coords == get_board_coords(self.en_passant_square.clone()) {
            match self.active_player {
                'w' => self.set_piece(&vec![target_coords[0]+1, target_coords[1]], '*'),
                'b' => self.set_piece(&vec![target_coords[0]-1, target_coords[1]], '*'),
                _ => panic!("ACTIVE PLAYER DOES NOT EXIST")
            }
            // TODO: remove the pawn at (y-1 if active player is black) and at (y+1 if active player is white)
        }
    }
} // Moves a piece to a target square.
    fn set_piece(&mut self, coords: &Vec<i32>, piece: char) {
        self.board_state[coords[0] as usize][coords[1] as usize] = piece;
    } // changes the given board coordinate to the given piece. 
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
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_bishop_moves() {
        let test_position = Game::new_from_fen("8/8/8/8/8/8/8/6B1 w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_queen_moves() {
        let test_position = Game::new_from_fen("p7/5p2/3P4/1P1Q3P/4p3/1p6/3P4/8 w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_knight_moves() {
        let test_position = Game::new_from_fen("N6N/8/8/4N3/8/8/8/N6N w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test] //manual test
    fn test_king_moves() {
        let test_position = Game::new_from_fen("K6K/8/3pR3/3K4/8/8/8/K6K w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)
    }
    #[test]
    fn test_for_checks() {
        let test_position_1 = Game::new();
        let test_position_2 = Game::new_from_fen("2k1r3/ppp2p1p/5p2/5P2/1P6/1n4P1/2R3BP/2K5 w - - 1 24".to_string());
        println!("{:?}", test_position_1);
        println!("{:?}", get_available_moves(test_position_1.board.clone(), test_position_1.board.active_player, false));
        println!("{:?}", test_position_2);
        println!("{:?}", get_available_moves(test_position_2.board.clone(), test_position_2.board.active_player, false));
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
        let test_position = Game::new_from_fen("8/8/8/8/8/3P1p2/1P3PP1/8 w - - 0 1".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        assert_eq!(true, true)  
    }
    #[test]
    fn test_en_passant() {
        let mut test_position = Game::new_from_fen("4k3/6p1/8/pP1pP3/7P/8/8/4K3 w - d6 0 6".to_string());
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        let result = test_position.make_move("e5".to_string(), "d6".to_string());
        
        println!("{:?}", test_position);
        println!("{:?}", get_available_moves(test_position.board.clone(), test_position.board.active_player, false));
        debug_assert!(result);
    }
}
