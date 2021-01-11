use crate::chess;
use std::str::{FromStr, Split};

#[derive(Debug)]
pub struct GoOptions {
    search_moves: Option<Vec<chess::Movement>>,

    white_time: Option<u64>,
    black_time: Option<u64>,
    white_increment: Option<u32>,
    black_increment: Option<u32>,
    moves_to_go: Option<u8>,

    depth: Option<u8>,
    nodes: Option<u64>,
    mate: Option<u8>,

    move_time: Option<u32>,
}

impl GoOptions {
    fn empty() -> GoOptions {
        GoOptions {
            search_moves: None,

            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,

            depth: None,
            nodes: None,
            mate: None,

            move_time: None,
        }
    }
}

#[derive(Debug)]
pub enum GoInstruction {
    Ponder,
    Infinite,
}

#[derive(Debug)]
pub struct Go {
    instruction: GoInstruction,
    option: GoOptions,
}

impl Go {
    fn new(instruction: GoInstruction) -> Go {
        Go {
            instruction,
            option: GoOptions::empty(),
        }
    }
}

#[derive(Debug)]
pub enum EngineMessage {
    UCI,
    Debug(bool),
    IsReady,

    UCINewGame,
    Position(chess::Board, Vec<chess::Movement>),
    Go(Go),

    Stop,
    PonderHit,
    Quit,

    DontMissTheShredderChessAnnualBarbeque, // Very important 10/10
}

fn get_moves(mut words: Split<char>) -> Option<Vec<chess::Movement>> {
    let mut moves = Vec::new();

    while let Some(word) = words.next() {
        moves.push(chess::Movement::from_notation(word)?);
    }

    Some(moves)
}

pub fn parse(s: &str) -> Option<EngineMessage> {
    let mut words = s.split(' ');

    Some(match words.next()? {
        "uci" => EngineMessage::UCI,
        "debug" => match words.next()? {
            "true" => EngineMessage::Debug(true),
            "false" => EngineMessage::Debug(false),
            _ => return None,
        },
        "isready" => EngineMessage::IsReady,

        "ucinewgame" => EngineMessage::UCINewGame,
        "position" => match words.next()? {
            // TODO: Clean up the duplication here
            "startpos" => {
                loop {
                    let word = words.next();
                    match word {
                        Some("moves") => break,
                        None => return None,
                        _ => continue,
                    }
                }

                EngineMessage::Position(chess::Board::from_start_pos(), get_moves(words)?)
            }

            "fen" => {
                let mut fen = Vec::new();
                loop {
                    let word = words.next();
                    match word {
                        Some("moves") => break,
                        Some(chunk) => fen.push(chunk),
                        None => return None,
                    }
                }
                let fen = fen.join(" ");

                EngineMessage::Position(chess::Board::from_fen(&fen)?, get_moves(words)?)
            }

            _ => return None,
        },
        "go" => {
            let mut go: Option<Go> = None;

            while let Some(word) = words.next() {
                match word {
                    "ponder" => go = Some(Go::new(GoInstruction::Ponder)),
                    "infinite" => go = Some(Go::new(GoInstruction::Infinite)),

                    "searchmoves" => {
                        let mut moves = Vec::new();
                        while let Some(word) = words.next() {
                            moves.push(chess::Movement::from_notation(word)?);
                        }

                        go.as_mut()?.option.search_moves = Some(moves);
                        break;
                    }

                    "wtime" => go.as_mut()?.option.white_time = u64::from_str(words.next()?).ok(),
                    "btime" => go.as_mut()?.option.black_time = u64::from_str(words.next()?).ok(),
                    "winc" => {
                        go.as_mut()?.option.white_increment = u32::from_str(words.next()?).ok()
                    }
                    "binc" => {
                        go.as_mut()?.option.black_increment = u32::from_str(words.next()?).ok()
                    }
                    "movestogo" => {
                        go.as_mut()?.option.moves_to_go = u8::from_str(words.next()?).ok()
                    }

                    "depth" => go.as_mut()?.option.depth = u8::from_str(words.next()?).ok(),
                    "nodes" => go.as_mut()?.option.nodes = u64::from_str(words.next()?).ok(),
                    "mate" => go.as_mut()?.option.mate = u8::from_str(words.next()?).ok(),

                    "movetime" => go.as_mut()?.option.move_time = u32::from_str(words.next()?).ok(),

                    _ => (),
                }
            }

            EngineMessage::Go(go?)
        }

        "stop" => EngineMessage::Stop,
        "ponderhit" => EngineMessage::PonderHit,
        "quit" => EngineMessage::Quit,

        "uwu" => EngineMessage::DontMissTheShredderChessAnnualBarbeque,

        _ => return None,
    })
}
