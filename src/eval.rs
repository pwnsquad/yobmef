use crate::chess::{Board, Color, Piece, Square};

// SHITTY SHIT HERE, JUST FOR EXPERIMENTATION, NOT FOR USE IN FINAL PROGRAM
// Inspiration from:
// https://www.chess.com/article/view/the-evaluation-of-material-imbalances-by-im-larry-kaufman
// https://www.chessprogramming.org/Simplified_Evaluation_Function

#[rustfmt::skip]
const PAWN_VALUE_TABLE: [i16; 64] = [
    0,  0,  0,   0,   0,   0,  0,  0,
    5,  10, 10, -20, -20,  10, 10, 5,
    5, -5, -10,  0,   0,  -10, -5,  5,
    0,  0,  0,   20,  20,  0,   0,  0,
    5,  5,  10,  25,  25,  10,  5,  5,
    10, 10, 20,  30,  30,  20,  10, 10,
    50, 50, 50,  50,  50,  50,  50, 50,
    0,  0,  0,   0,   0,   0,   0,  0,
];

#[rustfmt::skip]
const KNIGHT_VALUE_TABLE: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,  0,   5,   5,   0,  -20, -40,
    -30,  5,   10,  15,  15,  10,  5,  -30,
    -30,  0,   15,  20,  20,  15,  0,  -30,
    -30,  5,   15,  20,  20,  15,  5,  -30,
    -30,  0,   10,  15,  15,  10,  0,  -30,
    -40, -20,  0,   0,   0,   0,  -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const BISHOP_VALUE_TABLE: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,  5,   0,   0,   0,   0,   5,  -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,  0,   10,  10,  10,  10,  0,  -10,
    -10,  5,   5,   10,  10,  5,   5,  -10,
    -10,  0,   5,   10,  10,  5,   0,  -10,
    -10,  0,   0,   0,   0,   0,   0,  -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_VALUE_TABLE: [i16; 64] = [
    -5,    0,   0,   5,  5,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
     5,    10,  10,  10, 10,  10,  10,  5,
     0,    0,   0,   0,  0,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_VALUE_TABLE: [i16; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10,  0,   5,   0,  0,  0,   0,  -10,
    -10,  5,   5,   5,  5,  5,   0,  -10,
     0,   0,   5,   5,  5,  5,   0,  -5,
    -5,   0,   5,   5,  5,  5,   0,  -5,
    -10,  0,   5,   5,  5,  5,   0,  -10,
    -10,  0,   0,   0,  0,  0,   0,  -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_VALUE_TABLE_MIDDLEGAME: [i16; 64] = [
     20,  200, 180,  0,   0,   10,  180,  20,
     20,  20,  0,    0,   0,   0,   20,   20,
    -10, -20, -20,  -20, -20, -20, -20,  -10,
    -20, -30, -30,  -40, -40, -30, -30,  -20,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
];

fn get_score_for_piece(board: &Board, color: Color, piece: Piece) -> i16 {
    let value = match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 975,
        Piece::King => 0,
    };
    let table = match piece {
        Piece::Pawn => PAWN_VALUE_TABLE,
        Piece::Knight => KNIGHT_VALUE_TABLE,
        Piece::Bishop => BISHOP_VALUE_TABLE,
        Piece::Rook => ROOK_VALUE_TABLE,
        Piece::Queen => QUEEN_VALUE_TABLE,
        Piece::King => KING_VALUE_TABLE_MIDDLEGAME,
    };

    let bitboard = board.pieces(piece);
    let mut bitboard = *bitboard & *board.color_combined(color);
    if color == Color::Black {
        bitboard.flip_vertical_mut()
    };

    (0..64)
        .map(|i| {
            let exists = bitboard.get(Square(i)) as i16;
            let offset = table[i as usize];
            let offset_value = (value as i16) + offset;

            exists * offset_value
        })
        .sum()
}

// Not i16::MAX, because we use i16::MAX as infinity, ie.
// we want best move updated from None -> Some(mv) even if
// the best move still results in our demise.
pub const MATE: i16 = 10000;

fn get_score_for_color(board: &Board, color: Color) -> i16 {
    let mut score = 0;
    score += get_score_for_piece(board, color, Piece::Pawn);
    score += get_score_for_piece(board, color, Piece::Knight);
    score += get_score_for_piece(board, color, Piece::Bishop);
    score += get_score_for_piece(board, color, Piece::Rook);
    score += get_score_for_piece(board, color, Piece::Queen);
    score += get_score_for_piece(board, color, Piece::King);
    score
}

pub fn get_score_ongoing(board: &Board) -> i16 {
    get_score_for_color(board, Color::White) - get_score_for_color(&board, Color::Black)
}

pub fn get_score(board: &Board, legal_move_count: usize) -> i16 {
    if legal_move_count == 0 && board.in_check() {
        MATE * board.side_to_move.other().polarize()
    } else if legal_move_count > 0 {
        get_score_ongoing(board)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::gen_moves_once;
    use crate::{chess::Movement, movegen};

    #[test]
    fn test_get_score_e2e4() {
        gen_moves_once();

        let mut b = Board::from_start_pos();
        b.make_move_mut(&Movement::from_notation("e2e4").expect("e2e4 move is valid"));

        let score = get_score(&b, movegen::get_legal_moves(&b).len());
        eprintln!("score: {}", score);
        assert!(score > 0); // White should have the advantage
    }

    #[test]
    fn test_get_score_mate_for_black() {
        gen_moves_once();

        let b =
            Board::from_fen("r1b1kb1r/pppp1pp1/2n5/1B2p3/4PP2/6p1/PPPP2Pq/RNBQNRK1 w kq f3 0 8")
                .unwrap();
        let score = get_score(&b, movegen::get_legal_moves(&b).len());

        eprintln!("board:\n{}", b);
        eprintln!("score (white in checkmate) = {}", score);
        assert_eq!(score, -MATE);
    }

    #[test]
    fn test_get_score_mate_for_white() {
        gen_moves_once();

        let b = Board::from_fen("k1R5/8/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        let score = get_score(&b, movegen::get_legal_moves(&b).len());

        eprintln!("board:\n{}", b);
        eprintln!("score (black in checkmate) = {}", score);
        assert_eq!(score, MATE);
    }

    #[test]
    fn test_get_score_castle() {
        gen_moves_once();

        let mut b = Board::from_fen("rnbqkb1r/ppp2ppp/3p1n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4").unwrap();
        let score_1 = get_score(&b, movegen::get_legal_moves(&b).len());
        b.make_move_mut(&Movement::from_notation("e1g1").unwrap());
        let score_2 = get_score(&b, movegen::get_legal_moves(&b).len());
        
        println!("{} should be > {}", score_2, score_1);
        assert!(score_2 > score_1);
    }
}
