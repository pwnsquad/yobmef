use crate::chess::Piece;
use crate::chess::Square;

// Calling it Movement and not Move because "move" is a keyword
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Movement {
    pub from_square: Square,
    pub to_square: Square,
    pub promote: Option<Piece>,
}

impl Movement {
    pub fn new(from_square: Square, to_square: Square, promote: Option<Piece>) -> Movement {
        Movement {
            from_square,
            to_square,
            promote,
        }
    }

    pub fn from_notation(lan: &str) -> Option<Movement> {
        let from_square = Square::from_notation(&lan.get(0..2)?)?;
        let to_square = Square::from_notation(&lan.get(2..4)?)?;
        let promote = lan.chars().nth(4).and_then(|ch| Piece::from_char(ch));

        Some(Movement {
            from_square,
            to_square,
            promote,
        })
    }

    pub fn to_notation(&self) -> String {
        let from_notation = self.from_square.to_notation();
        let to_notation = self.to_square.to_notation();

        let mut lan = String::new();
        lan.push(from_notation.1);
        lan.push(from_notation.0);
        lan.push(to_notation.1);
        lan.push(to_notation.0);

        if let Some(piece) = self.promote {
            lan.push(piece.as_char());
        }

        lan
    }
}