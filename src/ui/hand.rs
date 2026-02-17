use eframe::egui::{self, Align, Color32, Layout, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use shogi::{Color, Piece, PieceType};

use crate::core::game::{GameState, piece_label};
use crate::ui::assets::{UiAssets, piece_asset_key};

const HAND_GRID_COLS: usize = 3;
const HAND_GRID_ROWS: usize = 3;
const HAND_GRID_GAP: f32 = 6.0;
const HAND_INNER_PAD: f32 = 8.0;
pub const HAND_PANEL_SIZE: f32 = 200.0;
const HAND_GRID_SLOTS: [Option<PieceType>; HAND_GRID_COLS * HAND_GRID_ROWS] = [
    Some(PieceType::Rook),
    Some(PieceType::Bishop),
    Some(PieceType::Gold),
    Some(PieceType::Silver),
    Some(PieceType::Knight),
    Some(PieceType::Lance),
    None,
    Some(PieceType::Pawn),
    None,
];

pub struct HandUiOutput {
    pub clicked_piece: Option<PieceType>,
    pub drag_started_piece: Option<PieceType>,
}

#[derive(Clone, Copy)]
pub enum HandPanelAnchor {
    Top,
    Bottom,
}

pub fn draw_hand_panel(
    ui: &mut egui::Ui,
    game: &GameState,
    assets: &UiAssets,
    color: Color,
    title: &str,
    anchor: HandPanelAnchor,
    hidden_piece: Option<PieceType>,
) -> HandUiOutput {
    let (panel_rect, _) =
        ui.allocate_exact_size(vec2(HAND_PANEL_SIZE, HAND_PANEL_SIZE), Sense::hover());
    let painter = ui.painter_at(panel_rect);
    if let Some(board_tex) = &assets.board_texture {
        painter.image(
            board_tex.id(),
            panel_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(panel_rect, 4.0, Color32::from_rgb(237, 206, 141));
    }
    painter.rect_stroke(
        panel_rect,
        4.0,
        Stroke::new(1.0, Color32::from_black_alpha(70)),
        StrokeKind::Outside,
    );

    let inner_rect = panel_rect.shrink(HAND_INNER_PAD);
    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::top_down(Align::Center)),
    );
    content.label(title);
    content.add_space(6.0);

    let grid_h = inner_rect.height() - 28.0;
    if matches!(anchor, HandPanelAnchor::Bottom) {
        let push_down = (content.available_height() - grid_h).max(0.0);
        content.add_space(push_down);
    }

    draw_hand_grid(&mut content, game, assets, color, hidden_piece)
}

fn draw_hand_grid(
    ui: &mut egui::Ui,
    game: &GameState,
    assets: &UiAssets,
    color: Color,
    hidden_piece: Option<PieceType>,
) -> HandUiOutput {
    let side_to_move = game.side_to_move();
    let interactive = side_to_move == color && !game.has_pending_promotion();
    let selected = game.selected_hand_piece();
    let mut clicked = None;
    let mut drag_started = None;

    let grid_w = ui.available_width();
    let grid_h = ui.available_height();
    let cell_w = (grid_w - HAND_GRID_GAP * (HAND_GRID_COLS as f32 - 1.0)) / HAND_GRID_COLS as f32;
    let cell_h = (grid_h - HAND_GRID_GAP * (HAND_GRID_ROWS as f32 - 1.0)) / HAND_GRID_ROWS as f32;
    let cell_size = cell_w.min(cell_h).max(36.0);
    let used_w = cell_size * HAND_GRID_COLS as f32 + HAND_GRID_GAP * (HAND_GRID_COLS as f32 - 1.0);
    let used_h = cell_size * HAND_GRID_ROWS as f32 + HAND_GRID_GAP * (HAND_GRID_ROWS as f32 - 1.0);
    let (grid_rect, _) = ui.allocate_exact_size(vec2(used_w, used_h), Sense::hover());

    for row in 0..HAND_GRID_ROWS {
        for col in 0..HAND_GRID_COLS {
            let idx = row * HAND_GRID_COLS + col;
            let Some(piece_type) = HAND_GRID_SLOTS[idx] else {
                continue;
            };

            let cell_rect = Rect::from_min_size(
                pos2(
                    grid_rect.min.x + col as f32 * (cell_size + HAND_GRID_GAP),
                    grid_rect.min.y + row as f32 * (cell_size + HAND_GRID_GAP),
                ),
                vec2(cell_size, cell_size),
            );

            let output = draw_hand_cell(
                ui,
                game,
                assets,
                color,
                interactive,
                selected,
                piece_type,
                hidden_piece,
                cell_rect,
            );
            clicked = clicked.or(output.0);
            drag_started = drag_started.or(output.1);
        }
    }

    HandUiOutput {
        clicked_piece: clicked,
        drag_started_piece: drag_started,
    }
}

fn draw_hand_cell(
    ui: &mut egui::Ui,
    game: &GameState,
    assets: &UiAssets,
    color: Color,
    interactive: bool,
    selected: Option<PieceType>,
    piece_type: PieceType,
    hidden_piece: Option<PieceType>,
    rect: Rect,
) -> (Option<PieceType>, Option<PieceType>) {
    let mut count = game.hand_count(color, piece_type);
    if hidden_piece == Some(piece_type) && game.side_to_move() == color && count > 0 {
        count -= 1;
    }
    let is_selected = interactive && selected == Some(piece_type);
    let can_click = interactive && count > 0;

    let color_id = if color == Color::Black { "b" } else { "w" };
    let response = ui.interact(
        rect,
        ui.id().with(("hand", color_id, piece_type as u8)),
        Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);

    let bg = if is_selected {
        Color32::from_rgba_unmultiplied(255, 231, 148, 160)
    } else if count == 0 {
        Color32::from_rgba_unmultiplied(255, 255, 255, 12)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 30)
    };
    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, Color32::from_black_alpha(50)),
        StrokeKind::Outside,
    );

    let piece = Piece { piece_type, color };
    let key = piece_asset_key(piece);
    let piece_rect = Rect::from_min_max(
        pos2(rect.min.x + 4.0, rect.min.y + 2.0),
        pos2(rect.max.x - 4.0, rect.max.y - 14.0),
    );
    if let Some(texture) = assets.piece_textures.get(&key) {
        painter.image(
            texture.id(),
            piece_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.text(
            piece_rect.center(),
            egui::Align2::CENTER_CENTER,
            piece_label(piece),
            egui::FontId::proportional(18.0),
            Color32::BLACK,
        );
    }

    if count > 0 {
        let badge_rect = Rect::from_min_max(
            pos2(rect.max.x - 19.0, rect.max.y - 16.0),
            pos2(rect.max.x - 3.0, rect.max.y - 2.0),
        );
        painter.rect_filled(badge_rect, 3.0, Color32::from_rgb(35, 35, 35));
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{count}"),
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );
    }

    if !interactive {
        painter.rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(0, 0, 0, 64));
    } else if count == 0 {
        painter.rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(0, 0, 0, 72));
    }

    let clicked = can_click && response.clicked();
    let drag_started = can_click && response.drag_started();
    (
        clicked.then_some(piece_type),
        drag_started.then_some(piece_type),
    )
}
