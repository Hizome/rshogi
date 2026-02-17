use eframe::egui::{self, Align2, Color32, Rect, pos2};

use crate::app::action::Action;
use crate::core::game::{GameState, piece_type_label, promoted_piece_type};
use crate::ui::assets::{UiAssets, piece_asset_key};

pub fn draw_promotion_dialog(
    ctx: &egui::Context,
    game: &GameState,
    assets: &UiAssets,
) -> Option<Action> {
    if !game.has_pending_promotion() {
        return None;
    }

    let mut pending_action = None;
    egui::Window::new("Promotion")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Choose promotion");
            ui.add_space(4.0);

            if let Some(piece) = game.pending_promotion_piece() {
                ui.label(format!(
                    "Current piece: {}",
                    piece_type_label(piece.piece_type)
                ));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    draw_choice_card(
                        ui,
                        assets,
                        piece,
                        true,
                        "Promote",
                        &mut pending_action,
                        Action::ChoosePromotion(true),
                    );
                    draw_choice_card(
                        ui,
                        assets,
                        piece,
                        false,
                        "Do not promote",
                        &mut pending_action,
                        Action::ChoosePromotion(false),
                    );
                });
            } else {
                ui.horizontal(|ui| {
                    if ui.button("Promote").clicked() {
                        pending_action = Some(Action::ChoosePromotion(true));
                    }
                    if ui.button("Do not promote").clicked() {
                        pending_action = Some(Action::ChoosePromotion(false));
                    }
                });
            }

            ui.add_space(6.0);
            if ui.button("Cancel").clicked() {
                pending_action = Some(Action::CancelPromotion);
            }
        });

    pending_action
}

fn draw_choice_card(
    ui: &mut egui::Ui,
    assets: &UiAssets,
    piece: shogi::Piece,
    promote: bool,
    label: &str,
    pending_action: &mut Option<Action>,
    action: Action,
) {
    ui.vertical(|ui| {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(104.0, 112.0), egui::Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(
            rect,
            6.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 26),
        );
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, Color32::from_black_alpha(70)),
            egui::StrokeKind::Outside,
        );

        let display_piece = if promote {
            shogi::Piece {
                piece_type: promoted_piece_type(piece.piece_type),
                color: piece.color,
            }
        } else {
            piece
        };
        let key = piece_asset_key(display_piece);
        if let Some(texture) = assets.piece_textures.get(&key) {
            let piece_rect = Rect::from_min_max(
                pos2(rect.min.x + 14.0, rect.min.y + 8.0),
                pos2(rect.max.x - 14.0, rect.min.y + 72.0),
            );
            painter.image(
                texture.id(),
                piece_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        painter.text(
            pos2(rect.center().x, rect.max.y - 16.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        if response.clicked() {
            *pending_action = Some(action);
        }
    });
}
