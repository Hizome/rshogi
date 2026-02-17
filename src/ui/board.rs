use eframe::egui::{self, Color32, Rect, Sense, pos2, vec2};
use shogi::{Piece, Square};

use crate::core::game::{GameState, piece_label};
use crate::ui::assets::{UiAssets, piece_asset_key};

const BOARD_SIZE: usize = 9;
const CELL_SIZE: f32 = 56.0;

pub struct BoardUiOutput {
    pub clicked_square: Option<Square>,
    pub drag_started: Option<(Square, Piece)>,
    pub board_rect: Rect,
}

pub fn board_pixel_size() -> f32 {
    CELL_SIZE * BOARD_SIZE as f32
}

pub fn draw_board(
    ui: &mut egui::Ui,
    game: &GameState,
    assets: &UiAssets,
    hidden_square: Option<Square>,
) -> BoardUiOutput {
    let board_size = board_pixel_size();
    let (board_rect, _) = ui.allocate_exact_size(vec2(board_size, board_size), Sense::hover());
    let painter = ui.painter_at(board_rect);

    paint_board_texture(&painter, board_rect, assets);
    paint_board_grid(&painter, board_rect);

    let mut clicked_square = None;
    let mut drag_started = None;
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let cell_rect = Rect::from_min_size(
                pos2(
                    board_rect.min.x + col as f32 * CELL_SIZE,
                    board_rect.min.y + row as f32 * CELL_SIZE,
                ),
                vec2(CELL_SIZE, CELL_SIZE),
            );
            let sq = square_from_ui(row, col);
            let piece = if Some(sq) == hidden_square {
                None
            } else {
                game.piece_at(sq)
            };

            paint_square_overlay(&painter, cell_rect, game, sq);

            let response = ui.interact(
                cell_rect,
                ui.id().with(("sq", row, col)),
                Sense::click_and_drag(),
            );
            if response.clicked() {
                clicked_square = Some(sq);
            }
            if response.drag_started() && piece.is_some_and(|p| p.color == game.side_to_move()) {
                drag_started = piece.map(|p| (sq, p));
            }

            if let Some(piece) = piece {
                paint_piece(&painter, cell_rect, piece, assets);
            }
        }
    }

    BoardUiOutput {
        clicked_square,
        drag_started,
        board_rect,
    }
}

fn paint_board_texture(painter: &egui::Painter, board_rect: Rect, assets: &UiAssets) {
    if let Some(board_tex) = &assets.board_texture {
        painter.image(
            board_tex.id(),
            board_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(board_rect, 0.0, Color32::from_rgb(237, 206, 141));
    }
}

fn paint_board_grid(painter: &egui::Painter, board_rect: Rect) {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let cell_rect = Rect::from_min_size(
                pos2(
                    board_rect.min.x + col as f32 * CELL_SIZE,
                    board_rect.min.y + row as f32 * CELL_SIZE,
                ),
                vec2(CELL_SIZE, CELL_SIZE),
            );
            painter.rect_stroke(
                cell_rect,
                0.0,
                egui::Stroke::new(1.0, Color32::from_black_alpha(40)),
                egui::StrokeKind::Outside,
            );
        }
    }
}

fn paint_square_overlay(painter: &egui::Painter, cell_rect: Rect, game: &GameState, sq: Square) {
    let highlight = if Some(sq) == game.selected() {
        Some(Color32::from_rgba_unmultiplied(255, 231, 148, 110))
    } else if game.is_legal_destination(sq) {
        if game.is_drop_mode() {
            Some(Color32::from_rgba_unmultiplied(135, 185, 255, 120))
        } else {
            Some(Color32::from_rgba_unmultiplied(195, 228, 181, 110))
        }
    } else {
        None
    };

    if let Some(fill) = highlight {
        painter.rect_filled(cell_rect, 0.0, fill);
    }

    if game.is_drop_mode() && game.is_legal_destination(sq) {
        painter.circle_filled(cell_rect.center(), 4.0, Color32::from_rgb(52, 114, 232));
    }
}

fn paint_piece(painter: &egui::Painter, cell_rect: Rect, piece: Piece, assets: &UiAssets) {
    let key = piece_asset_key(piece);
    if let Some(texture) = assets.piece_textures.get(&key) {
        let piece_rect = cell_rect.shrink(1.5);
        painter.image(
            texture.id(),
            piece_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        let text = piece_label(piece);
        painter.text(
            cell_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(20.0),
            Color32::BLACK,
        );
    }
}

fn square_from_ui(row: usize, col: usize) -> Square {
    // UI uses left->right files 9..1, top->bottom ranks a..i.
    let file = (BOARD_SIZE - 1 - col) as u8;
    let rank = row as u8;
    Square::new(file, rank).expect("valid board coordinate")
}

pub fn square_at_pos(board_rect: Rect, pos: egui::Pos2) -> Option<Square> {
    if !board_rect.contains(pos) {
        return None;
    }
    let x = pos.x - board_rect.min.x;
    let y = pos.y - board_rect.min.y;
    let col = (x / CELL_SIZE).floor() as usize;
    let row = (y / CELL_SIZE).floor() as usize;
    if row >= BOARD_SIZE || col >= BOARD_SIZE {
        return None;
    }
    Some(square_from_ui(row, col))
}
