use crate::app::action::Action;
use crate::app::state::RShogiApp;
use crate::core::game::GameState;

pub fn reduce(app: &mut RShogiApp, action: Action) {
    reduce_game(&mut app.game, action);
}

pub fn reduce_game(game: &mut GameState, action: Action) {
    match action {
        Action::ClickSquare(sq) => game.on_square_clicked(sq),
        Action::SelectHandPiece(piece_type) => game.select_hand_piece(piece_type),
        Action::ChoosePromotion(promote) => game.choose_promotion(promote),
        Action::CancelPromotion => game.cancel_promotion(),
    }
}
