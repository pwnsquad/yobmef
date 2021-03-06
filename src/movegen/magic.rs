use rand::rngs::StdRng;
use rand::SeedableRng;

use super::magic_utils::{get_occupancy_mask, get_questions_and_answers, NUM_MOVES};

use crate::bitboard::BitBoard;
use crate::chess::{Board, Color, Movement, Piece, Square};

static mut MOVES: [BitBoard; NUM_MOVES] = [BitBoard::empty(); NUM_MOVES];
static mut ROOK_MAGICS: [MagicSquare; 64] = [MagicSquare::empty(); 64];
static mut BISHOP_MAGICS: [MagicSquare; 64] = [MagicSquare::empty(); 64];

#[derive(Debug, Clone, Copy)]
pub struct MagicSquare {
    number: BitBoard,
    occupancy_mask: BitBoard,
    offset: u32,
    right_shift: u8,
    empty: bool,
}

impl MagicSquare {
    fn new(
        number: BitBoard,
        occupancy_mask: BitBoard,
        offset: u32,
        right_shift: u8,
    ) -> MagicSquare {
        MagicSquare {
            number,
            occupancy_mask,
            offset,
            right_shift,
            empty: false,
        }
    }

    const fn empty() -> MagicSquare {
        MagicSquare {
            number: BitBoard::empty(),
            occupancy_mask: BitBoard::empty(),
            offset: 0,
            right_shift: 0,
            empty: true,
        }
    }

    pub fn lookup_hash(&self, occupancy: &BitBoard) -> BitBoard {
        if self.empty {
            panic!(
                "tried to lookup hash on an empty magic square, did you forget to run gen_moves?"
            );
        }

        let raw_hash = self.number * (self.occupancy_mask & occupancy);
        let shifted_hash = (raw_hash.0 as usize) >> (self.right_shift as usize);
        let hash = (self.offset as usize) + shifted_hash;

        unsafe { MOVES[hash] }
    }
}

static SEEDS: [u64; 8] = [8198, 15098, 15153, 12593, 16340, 19763, 55569, 7831];

// TODO: Go through, fully re-comprehend, and refactor this BS
fn gen_single_magic(from_sq: Square, piece: Piece, cur_offset: usize) -> usize {
    let (questions, answers) = get_questions_and_answers(from_sq, piece);

    let occupancy_mask = get_occupancy_mask(from_sq, piece);
    let mut new_magic = MagicSquare::new(
        BitBoard::empty(),
        occupancy_mask,
        cur_offset as u32,
        (questions.len().leading_zeros() + 1) as u8,
    );

    let mut done = false;
    let mut rng = StdRng::seed_from_u64(SEEDS[from_sq.rank() as usize]);

    while !done {
        let magic_bitboard = BitBoard::random(&mut rng);

        // DEAR GOD
        if (occupancy_mask * magic_bitboard).count_ones() < 6 {
            continue;
        }

        // AAAAAAAAAAAAAAAA
        let mut new_answers = vec![BitBoard::empty(); questions.len()];
        done = true;
        for i in 0..questions.len() {
            let hash = (magic_bitboard * questions[i]).0 >> (new_magic.right_shift as u64);
            let j = hash as usize;
            if new_answers[j] == BitBoard::empty() {
                new_answers[j] = answers[i];
            } else {
                done = false;
                break;
            }
        }

        if done {
            new_magic.number = magic_bitboard;
        }
    }

    unsafe {
        if piece == Piece::Rook {
            ROOK_MAGICS[from_sq.0 as usize] = new_magic;
        } else {
            BISHOP_MAGICS[from_sq.0 as usize] = new_magic;
        }

        for i in 0..questions.len() {
            let hash = (new_magic.number * questions[i]) >> (new_magic.right_shift as u64);
            let j = hash.0 as usize;
            MOVES[(new_magic.offset as usize) + j] = answers[i];
        }
    }

    cur_offset + questions.len()
}

pub fn gen_all_magics() {
    let mut cur_offset = 0;

    for sq_index in 0..64 {
        cur_offset = gen_single_magic(Square(sq_index), Piece::Bishop, cur_offset);
    }
    for sq_index in 0..64 {
        cur_offset = gen_single_magic(Square(sq_index), Piece::Rook, cur_offset);
    }
}

fn get_sliding_moves_bb(sq: Square, piece: Piece, occupancy: &BitBoard) -> BitBoard {
    unsafe {
        if piece == Piece::Rook {
            let magic = ROOK_MAGICS[sq.0 as usize];
            magic.lookup_hash(occupancy)
        } else if piece == Piece::Bishop {
            let magic = BISHOP_MAGICS[sq.0 as usize];
            magic.lookup_hash(occupancy)
        } else if piece == Piece::Queen {
            let bishop_magic = BISHOP_MAGICS[sq.0 as usize];
            let rook_magic = ROOK_MAGICS[sq.0 as usize];
            bishop_magic.lookup_hash(occupancy) | rook_magic.lookup_hash(occupancy)
        } else {
            panic!("{:?} is not a sliding piece", piece);
        }
    }
}

pub fn get_sliding_attacks(board: &Board, color: Color) -> BitBoard {
    let mut attacks = BitBoard::empty();

    let all_pieces = board.combined();
    let my_pieces = *board.color_combined(color);

    let my_queens = *board.pieces(Piece::Queen) & my_pieces;
    let my_rooks = *board.pieces(Piece::Rook) & my_pieces;
    let my_bishops = *board.pieces(Piece::Bishop) & my_pieces;

    for from_sq_index in 0..64 {
        let from_sq = Square(from_sq_index);
        if my_rooks.get(from_sq) {
            attacks |= get_sliding_moves_bb(from_sq, Piece::Rook, &all_pieces)
        } else if my_bishops.get(from_sq) {
            attacks |= get_sliding_moves_bb(from_sq, Piece::Bishop, &all_pieces)
        } else if my_queens.get(from_sq) {
            attacks |= get_sliding_moves_bb(from_sq, Piece::Queen, &all_pieces)
        }
    }

    attacks
}

pub fn get_sliding_moves(board: &Board, moves: &mut Vec<Movement>, color: Color) {
    let all_pieces = board.combined();
    let my_pieces = *board.color_combined(color);

    let my_queens = *board.pieces(Piece::Queen) & my_pieces;
    let my_rooks = *board.pieces(Piece::Rook) & my_pieces;
    let my_bishops = *board.pieces(Piece::Bishop) & my_pieces;
    let my_sliding_pieces = my_rooks | my_bishops | my_queens;

    for from_sq in my_sliding_pieces {
        let moves_bitboard = if my_rooks.get(from_sq) {
            get_sliding_moves_bb(from_sq, Piece::Rook, &all_pieces) & !my_pieces
        } else if my_bishops.get(from_sq) {
            get_sliding_moves_bb(from_sq, Piece::Bishop, &all_pieces) & !my_pieces
        } else if my_queens.get(from_sq) {
            get_sliding_moves_bb(from_sq, Piece::Queen, &all_pieces) & !my_pieces
        } else {
            panic!("my_sliding_pieces contains a non sliding piece");
        };

        for to_sq in moves_bitboard {
            moves.push(Movement::new(from_sq, to_sq, None));
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::movegen::helpers::{bitboard_test, moves_test};

    #[test]
    fn test_rook_move_lookup() {
        gen_all_magics();
        let sq = Square::from_notation("d5").unwrap();

        let mut occupancy = BitBoard::empty();
        occupancy.flip_mut(Square::from_notation("d3").unwrap());
        occupancy.flip_mut(Square::from_notation("h5").unwrap());

        let moves = get_sliding_moves_bb(sq, Piece::Rook, &occupancy);
        bitboard_test(&moves, "f5 h5 d3 b5 d6 d7 d8", "d2 d1 g3");
    }

    #[test]
    fn test_bishop_move_lookup() {
        gen_all_magics();
        let sq = Square::from_notation("g3").unwrap();

        let mut occupancy = BitBoard::empty();
        occupancy.flip_mut(Square::from_notation("f2").unwrap());
        occupancy.flip_mut(Square::from_notation("e5").unwrap());

        let moves = get_sliding_moves_bb(sq, Piece::Bishop, &occupancy);
        bitboard_test(&moves, "f4 e5 h4 h2 f2", "e1 g3 d6 c7 b8");
    }

    #[test]
    fn test_get_all_sliding_moves() {
        let board = Board::from_fen("4K3/7k/Q4b2/8/6p1/R7/4B3/8 w - - 0 1").unwrap();

        // Rook moves
        moves_test(&board, "a3e3 a3h3 a3a2 a3a5", "a3a6 a3a8");

        // Bishop moves
        moves_test(&board, "e2g4 e2b5 e2d1", "e2h5 e2a6");

        // Queen moves
        moves_test(&board, "a6d6 a6c8 a6a4 a6d3", "a6g6 a6e2 a6a3 a6a2");
    }
}
