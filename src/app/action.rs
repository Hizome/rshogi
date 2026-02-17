use shogi::{PieceType, Square};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    ClickSquare(Square),
    SelectHandPiece(PieceType),
    ChoosePromotion(bool),
    CancelPromotion,
}
