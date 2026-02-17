use eframe::egui::{self, Color32, Rect, pos2, vec2};
use shogi::{Color, Piece, PieceType, Square};

use crate::app::action::Action;
use crate::app::update::reduce;
use crate::core::game::{GameState, piece_label, piece_type_label};
use crate::ui::assets::{self, UiAssets, piece_asset_key};
use crate::ui::board;
use crate::ui::hand;
use crate::ui::promotion;

#[derive(Default)]
pub struct RShogiApp {
    pub game: GameState,
    pub assets: UiAssets,
    pub assets_loaded: bool,
    pub assets_error: String,
    drag: Option<DragState>,
}

impl RShogiApp {
    pub fn new() -> Self {
        Self {
            game: GameState::new(),
            assets: UiAssets::default(),
            assets_loaded: false,
            assets_error: String::new(),
            drag: None,
        }
    }

    fn dispatch(&mut self, action: Action) {
        reduce(self, action);
    }

    fn ensure_assets_loaded(&mut self, ctx: &egui::Context) {
        if self.assets_loaded {
            return;
        }
        self.assets_loaded = true;

        match assets::load_assets(ctx) {
            Ok(ui_assets) => self.assets = ui_assets,
            Err(err) => self.assets_error = err,
        }
    }
}

impl eframe::App for RShogiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_assets_loaded(ctx);

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Turn: {}", self.game.side_to_move()));
                ui.separator();
                ui.label(format!("Ply: {}", self.game.ply()));
            });
            if let Some(piece_type) = self.game.selected_hand_piece() {
                let count = self.game.hand_count(self.game.side_to_move(), piece_type);
                ui.label(format!(
                    "Drop mode: {} x{}",
                    piece_type_label(piece_type),
                    count
                ));
            }
            if !self.game.status().is_empty() {
                ui.colored_label(Color32::LIGHT_RED, self.game.status());
            }
            if !self.assets_error.is_empty() {
                ui.colored_label(Color32::YELLOW, &self.assets_error);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut opponent_hand_output = None;
            let mut your_hand_output = None;
            let mut board_output = None;
            let hidden_square = self.drag.and_then(|d| match d.source {
                DragSource::Board(from) => Some(from),
                DragSource::Hand(_) => None,
            });
            let hidden_hand_piece = self.drag.and_then(|d| match d.source {
                DragSource::Hand(piece_type) => Some(piece_type),
                DragSource::Board(_) => None,
            });

            let board_size = board::board_pixel_size();
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    vec2(hand::HAND_PANEL_SIZE, board_size),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        opponent_hand_output = Some(hand::draw_hand_panel(
                            ui,
                            &self.game,
                            &self.assets,
                            Color::White,
                            "Opponent hand",
                            hand::HandPanelAnchor::Top,
                            None,
                        ));
                    },
                );

                ui.add_space(10.0);
                board_output = Some(board::draw_board(
                    ui,
                    &self.game,
                    &self.assets,
                    hidden_square,
                ));
                ui.add_space(10.0);

                ui.allocate_ui_with_layout(
                    vec2(hand::HAND_PANEL_SIZE, board_size),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        let push_down = (board_size - hand::HAND_PANEL_SIZE).max(0.0);
                        ui.add_space(push_down);
                        your_hand_output = Some(hand::draw_hand_panel(
                            ui,
                            &self.game,
                            &self.assets,
                            Color::Black,
                            "Your hand",
                            hand::HandPanelAnchor::Bottom,
                            hidden_hand_piece,
                        ));
                    },
                );
            });

            if let (Some(opponent_hand_output), Some(your_hand_output), Some(board_output)) =
                (opponent_hand_output, your_hand_output, board_output)
            {
                let mut consumed = false;

                if self.drag.is_none() {
                    if let Some(piece_type) = your_hand_output.drag_started_piece {
                        self.game.preview_hand_drag_from(piece_type);
                        self.drag = Some(DragState {
                            source: DragSource::Hand(piece_type),
                            piece: Piece {
                                piece_type,
                                color: self.game.side_to_move(),
                            },
                        });
                        consumed = true;
                    } else if let Some((from, piece)) = board_output.drag_started {
                        self.game.preview_board_drag_from(from);
                        self.drag = Some(DragState {
                            source: DragSource::Board(from),
                            piece,
                        });
                        consumed = true;
                    }
                }

                let pointer_released = ctx.input(|i| i.pointer.any_released());
                if pointer_released {
                    if let Some(drag) = self.drag.take() {
                        let target = ctx
                            .input(|i| i.pointer.interact_pos())
                            .and_then(|pos| board::square_at_pos(board_output.board_rect, pos));

                        if let Some(to) = target {
                            match drag.source {
                                DragSource::Board(from) => self.game.perform_board_drag(from, to),
                                DragSource::Hand(piece_type) => {
                                    self.game.perform_hand_drag(piece_type, to)
                                }
                            }
                        } else {
                            self.game.clear_active_selection();
                        }
                        consumed = true;
                    }
                }

                if !consumed {
                    if let Some(piece_type) = opponent_hand_output.clicked_piece {
                        self.dispatch(Action::SelectHandPiece(piece_type));
                    }
                    if let Some(piece_type) = your_hand_output.clicked_piece {
                        self.dispatch(Action::SelectHandPiece(piece_type));
                    }
                    if let Some(sq) = board_output.clicked_square {
                        self.dispatch(Action::ClickSquare(sq));
                    }
                }
            }
        });

        if let Some(action) = promotion::draw_promotion_dialog(ctx, &self.game, &self.assets) {
            self.dispatch(action);
        }

        if let Some(drag) = self.drag {
            draw_drag_piece(ctx, &self.assets, drag.piece);
        }
    }
}

#[derive(Clone, Copy)]
struct DragState {
    source: DragSource,
    piece: Piece,
}

#[derive(Clone, Copy)]
enum DragSource {
    Board(Square),
    Hand(PieceType),
}

fn draw_drag_piece(ctx: &egui::Context, assets: &UiAssets, piece: Piece) {
    let pointer_pos = ctx.input(|i| i.pointer.interact_pos());
    let Some(pointer_pos) = pointer_pos else {
        return;
    };

    let size = 52.0;
    let rect = Rect::from_center_size(pointer_pos, egui::vec2(size, size));
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("drag-piece"),
    ));

    let key = piece_asset_key(piece);
    if let Some(texture) = assets.piece_textures.get(&key) {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.text(
            pointer_pos,
            egui::Align2::CENTER_CENTER,
            piece_label(piece),
            egui::FontId::proportional(24.0),
            Color32::WHITE,
        );
    }
}
