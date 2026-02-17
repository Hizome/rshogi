use std::collections::HashSet;

use shogi::{Color, Move, Piece, PieceType, Position, Square};

const START_SFEN: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
#[allow(dead_code)]
const BOARD_SIZE: u8 = 9;

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct PendingPromotion {
    promote: Move,
    non_promote: Move,
}

#[derive(Default)]
pub struct GameState {
    pos: Position,
    selected: Option<Square>,
    selected_hand: Option<PieceType>,
    legal_moves: Vec<Move>,
    legal_destinations: HashSet<Square>,
    pending_promotion: Option<PendingPromotion>,
    last_action_from: Option<Square>,
    last_action_to: Option<Square>,
    pending_sound: Option<SoundCue>,
    status: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCue {
    Move,
    Capture,
    Error,
}

#[allow(dead_code)]
impl GameState {
    pub fn new() -> Self {
        let mut pos = Position::new();
        let mut status = String::new();
        if let Err(err) = pos.set_sfen(START_SFEN) {
            status = format!("Failed to load initial SFEN: {err:?}");
        }

        Self {
            pos,
            selected: None,
            selected_hand: None,
            legal_moves: Vec::new(),
            legal_destinations: HashSet::new(),
            pending_promotion: None,
            last_action_from: None,
            last_action_to: None,
            pending_sound: None,
            status,
        }
    }

    pub fn side_to_move(&self) -> Color {
        self.pos.side_to_move()
    }

    pub fn ply(&self) -> u16 {
        self.pos.ply()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn selected(&self) -> Option<Square> {
        self.selected
    }

    pub fn selected_hand_piece(&self) -> Option<PieceType> {
        self.selected_hand
    }

    pub fn has_pending_promotion(&self) -> bool {
        self.pending_promotion.is_some()
    }

    pub fn last_action_from(&self) -> Option<Square> {
        self.last_action_from
    }

    pub fn last_action_to(&self) -> Option<Square> {
        self.last_action_to
    }

    pub fn take_pending_sound(&mut self) -> Option<SoundCue> {
        self.pending_sound.take()
    }

    pub fn is_drop_mode(&self) -> bool {
        self.selected_hand.is_some()
    }

    pub fn is_legal_destination(&self, sq: Square) -> bool {
        self.legal_destinations.contains(&sq)
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        *self.pos.piece_at(sq)
    }

    pub fn hand_count(&self, color: Color, piece_type: PieceType) -> u8 {
        self.pos.hand(Piece { piece_type, color })
    }

    pub fn select_hand_piece(&mut self, piece_type: PieceType) {
        if self.pending_promotion.is_some() {
            return;
        }

        let color = self.pos.side_to_move();
        if self.hand_count(color, piece_type) == 0 {
            self.clear_selection();
            return;
        }

        if self.selected_hand == Some(piece_type) {
            self.clear_selection();
            return;
        }

        self.selected = None;
        self.selected_hand = Some(piece_type);
        self.legal_moves = self.legal_drops_for(piece_type);
        self.legal_destinations = self
            .legal_moves
            .iter()
            .map(|m| match *m {
                Move::Normal { to, .. } | Move::Drop { to, .. } => to,
            })
            .collect();
    }

    pub fn on_square_clicked(&mut self, sq: Square) {
        if self.pending_promotion.is_some() {
            return;
        }

        let clicked_piece = *self.pos.piece_at(sq);
        let side_to_move = self.pos.side_to_move();

        if let Some(piece_type) = self.selected_hand {
            if self.legal_destinations.contains(&sq) {
                self.execute_move(Move::Drop { to: sq, piece_type });
                return;
            }

            if let Some(piece) = clicked_piece {
                if piece.color == side_to_move {
                    self.select_square(sq, piece);
                } else {
                    self.clear_selection();
                }
            } else {
                self.clear_selection();
            }
            return;
        }

        if let Some(selected_sq) = self.selected {
            if selected_sq == sq {
                self.clear_selection();
                return;
            }

            if self.legal_destinations.contains(&sq) {
                match self.pick_move_to(sq) {
                    MoveChoice::Single(chosen) => self.execute_move(chosen),
                    MoveChoice::NeedsPromotion {
                        promote,
                        non_promote,
                    } => {
                        self.pending_promotion = Some(PendingPromotion {
                            promote,
                            non_promote,
                        });
                    }
                    MoveChoice::None => {}
                }
                return;
            }
        }

        if let Some(piece) = clicked_piece {
            if piece.color == side_to_move {
                self.select_square(sq, piece);
            } else {
                self.clear_selection();
            }
        } else {
            self.clear_selection();
        }
    }

    pub fn choose_promotion(&mut self, promote: bool) {
        if let Some(pending) = self.pending_promotion.take() {
            let mv = if promote {
                pending.promote
            } else {
                pending.non_promote
            };
            self.execute_move(mv);
        }
    }

    pub fn cancel_promotion(&mut self) {
        self.pending_promotion = None;
    }

    pub fn clear_active_selection(&mut self) {
        self.clear_selection();
    }

    pub fn perform_board_drag(&mut self, from: Square, to: Square) {
        if self.pending_promotion.is_some() {
            return;
        }
        if from == to {
            self.clear_selection();
            return;
        }

        let piece = match *self.pos.piece_at(from) {
            Some(piece) if piece.color == self.pos.side_to_move() => piece,
            _ => {
                self.clear_selection();
                return;
            }
        };

        self.select_square(from, piece);
        if !self.legal_destinations.contains(&to) {
            self.clear_selection();
            return;
        }

        match self.pick_move_to(to) {
            MoveChoice::Single(chosen) => self.execute_move(chosen),
            MoveChoice::NeedsPromotion {
                promote,
                non_promote,
            } => {
                self.pending_promotion = Some(PendingPromotion {
                    promote,
                    non_promote,
                });
            }
            MoveChoice::None => self.clear_selection(),
        }
    }

    pub fn perform_hand_drag(&mut self, piece_type: PieceType, to: Square) {
        if self.pending_promotion.is_some() {
            return;
        }

        let color = self.pos.side_to_move();
        if self.hand_count(color, piece_type) == 0 {
            self.clear_selection();
            return;
        }

        self.selected = None;
        self.selected_hand = Some(piece_type);
        self.legal_moves = self.legal_drops_for(piece_type);
        self.legal_destinations = self
            .legal_moves
            .iter()
            .map(|m| match *m {
                Move::Normal { to, .. } | Move::Drop { to, .. } => to,
            })
            .collect();

        if self.legal_destinations.contains(&to) {
            self.execute_move(Move::Drop { to, piece_type });
        } else {
            self.clear_selection();
        }
    }

    pub fn preview_board_drag_from(&mut self, from: Square) {
        if self.pending_promotion.is_some() {
            return;
        }
        let piece = match *self.pos.piece_at(from) {
            Some(piece) if piece.color == self.pos.side_to_move() => piece,
            _ => {
                self.clear_selection();
                return;
            }
        };
        self.select_square(from, piece);
    }

    pub fn preview_hand_drag_from(&mut self, piece_type: PieceType) {
        self.select_hand_piece(piece_type);
    }

    pub fn pending_promotion_piece(&self) -> Option<Piece> {
        let pending = self.pending_promotion?;
        let from = match pending.non_promote {
            Move::Normal { from, .. } => from,
            Move::Drop { .. } => return None,
        };
        *self.pos.piece_at(from)
    }

    pub fn pending_promotion_target_square(&self) -> Option<Square> {
        let pending = self.pending_promotion?;
        match pending.non_promote {
            Move::Normal { to, .. } | Move::Drop { to, .. } => Some(to),
        }
    }

    fn select_square(&mut self, sq: Square, piece: Piece) {
        self.selected = Some(sq);
        self.selected_hand = None;
        self.legal_moves = self.legal_moves_from(sq, piece);
        self.legal_destinations = self
            .legal_moves
            .iter()
            .map(|m| match *m {
                Move::Normal { to, .. } | Move::Drop { to, .. } => to,
            })
            .collect();
    }

    fn clear_selection(&mut self) {
        self.selected = None;
        self.selected_hand = None;
        self.legal_moves.clear();
        self.legal_destinations.clear();
    }

    fn pick_move_to(&self, to: Square) -> MoveChoice {
        let mut promote = None;
        let mut non_promote = None;
        let mut drop_mv = None;

        for mv in &self.legal_moves {
            match *mv {
                Move::Drop { to: dst, .. } if dst == to => drop_mv = Some(*mv),
                Move::Normal {
                    to: dst,
                    promote: true,
                    ..
                } if dst == to => promote = Some(*mv),
                Move::Normal {
                    to: dst,
                    promote: false,
                    ..
                } if dst == to => non_promote = Some(*mv),
                _ => {}
            }
        }

        if let Some(mv) = drop_mv {
            return MoveChoice::Single(mv);
        }
        match (promote, non_promote) {
            (Some(p), Some(np)) => MoveChoice::NeedsPromotion {
                promote: p,
                non_promote: np,
            },
            (Some(p), None) => MoveChoice::Single(p),
            (None, Some(np)) => MoveChoice::Single(np),
            (None, None) => MoveChoice::None,
        }
    }

    fn legal_moves_from(&mut self, from: Square, piece: Piece) -> Vec<Move> {
        let mut out = Vec::new();
        let candidates = self.pos.move_candidates(from, piece);
        for to in candidates {
            let normal = Move::Normal {
                from,
                to,
                promote: false,
            };
            if self.try_move_legality(normal) {
                out.push(normal);
            }

            let promote = Move::Normal {
                from,
                to,
                promote: true,
            };
            if self.try_move_legality(promote) {
                out.push(promote);
            }
        }
        out
    }

    fn legal_drops_for(&mut self, piece_type: PieceType) -> Vec<Move> {
        let mut out = Vec::new();
        for file in 0..BOARD_SIZE {
            for rank in 0..BOARD_SIZE {
                let to = Square::new(file, rank).expect("valid board coordinate");
                if self.pos.piece_at(to).is_some() {
                    continue;
                }
                let drop_mv = Move::Drop { to, piece_type };
                if self.try_move_legality(drop_mv) {
                    out.push(drop_mv);
                }
            }
        }
        out
    }

    fn try_move_legality(&mut self, mv: Move) -> bool {
        if self.pos.make_move(mv).is_ok() {
            let _ = self.pos.unmake_move();
            true
        } else {
            false
        }
    }

    fn execute_move(&mut self, mv: Move) {
        let is_capture = match mv {
            Move::Normal { to, .. } => self.pos.piece_at(to).is_some(),
            Move::Drop { .. } => false,
        };
        self.pending_promotion = None;
        match self.pos.make_move(mv) {
            Ok(()) => {
                match mv {
                    Move::Normal { from, to, .. } => {
                        self.last_action_from = Some(from);
                        self.last_action_to = Some(to);
                    }
                    Move::Drop { to, .. } => {
                        self.last_action_from = None;
                        self.last_action_to = Some(to);
                    }
                }
                self.pending_sound = Some(if is_capture {
                    SoundCue::Capture
                } else {
                    SoundCue::Move
                });
                self.status.clear();
                self.clear_selection();
            }
            Err(err) => {
                self.pending_sound = Some(SoundCue::Error);
                self.status = format!("Move failed: {err:?}");
                self.clear_selection();
            }
        }
    }
}

enum MoveChoice {
    Single(Move),
    NeedsPromotion { promote: Move, non_promote: Move },
    None,
}

pub fn piece_label(piece: Piece) -> String {
    let base = match piece.piece_type {
        PieceType::King => "K",
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Gold => "G",
        PieceType::Silver => "S",
        PieceType::Knight => "N",
        PieceType::Lance => "L",
        PieceType::Pawn => "P",
        PieceType::ProRook => "+R",
        PieceType::ProBishop => "+B",
        PieceType::ProSilver => "+S",
        PieceType::ProKnight => "+N",
        PieceType::ProLance => "+L",
        PieceType::ProPawn => "+P",
    };

    if piece.color == Color::Black {
        base.to_string()
    } else {
        base.to_ascii_lowercase()
    }
}

#[allow(dead_code)]
pub fn piece_type_label(piece_type: PieceType) -> &'static str {
    match piece_type {
        PieceType::King => "King",
        PieceType::Rook => "Rook",
        PieceType::Bishop => "Bishop",
        PieceType::Gold => "Gold",
        PieceType::Silver => "Silver",
        PieceType::Knight => "Knight",
        PieceType::Lance => "Lance",
        PieceType::Pawn => "Pawn",
        PieceType::ProRook => "Dragon",
        PieceType::ProBishop => "Horse",
        PieceType::ProSilver => "Promoted Silver",
        PieceType::ProKnight => "Promoted Knight",
        PieceType::ProLance => "Promoted Lance",
        PieceType::ProPawn => "Tokin",
    }
}

#[allow(dead_code)]
pub fn promoted_piece_type(piece_type: PieceType) -> PieceType {
    match piece_type {
        PieceType::Rook => PieceType::ProRook,
        PieceType::Bishop => PieceType::ProBishop,
        PieceType::Silver => PieceType::ProSilver,
        PieceType::Knight => PieceType::ProKnight,
        PieceType::Lance => PieceType::ProLance,
        PieceType::Pawn => PieceType::ProPawn,
        other => other,
    }
}
