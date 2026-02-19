use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex, v_flex, *,
};
use shogi::{Color, Piece, PieceType, Square};

use crate::core::game::{piece_type_label, promoted_piece_type};
use crate::ui_gpui::assets::{board_asset_path, piece_asset_path};

use super::model::{
    BOARD_COORD_RIGHT_W, BOARD_SIZE, CELL_PX, DragSource, DragState, DrawCurrent, DrawShape,
    GpuiP1Shell, HAND_COL_W, HAND_PIECE_PX, HAND_PIECES, PIECE_PX, PROMO_CARD_H_RATIO,
    PROMO_CARD_RADIUS, PROMO_CARD_W_RATIO, PROMO_PIECE_PX, SCENE_GAP_PX,
};

impl GpuiP1Shell {
    fn render_promotion_overlay(&self, board_px: f32, cx: &mut Context<Self>) -> AnyElement {
        if !self.game.has_pending_promotion() {
            return div().into_any_element();
        }

        let Some(piece) = self.game.pending_promotion_piece() else {
            return div().into_any_element();
        };
        let Some(target_sq) = self.game.pending_promotion_target_square() else {
            return div().into_any_element();
        };

        let promoted = Piece {
            piece_type: promoted_piece_type(piece.piece_type),
            color: piece.color,
        };
        let (ui_row, ui_col) = Self::ui_pos_from_square(target_sq);
        let card_w = CELL_PX * PROMO_CARD_W_RATIO;
        let card_h = CELL_PX * PROMO_CARD_H_RATIO;
        let x = ui_col as f32 * CELL_PX + (CELL_PX - card_w) * 0.5;
        let raw_y = if piece.color == Color::White {
            (ui_row as f32 - 1.0) * CELL_PX
        } else {
            ui_row as f32 * CELL_PX
        };
        let y = raw_y.clamp(0.0, (board_px - card_h).max(0.0));
        let (top_piece, bottom_piece, top_promotes, bottom_promotes) =
            if piece.color == Color::White {
                (piece, promoted, false, true)
            } else {
                (promoted, piece, true, false)
            };

        let top_choice = div()
            .id("promote-choice-top")
            .w_full()
            .h(px(card_h / 2.0))
            .items_center()
            .justify_center()
            .child(
                img(piece_asset_path(top_piece, self.piece_wallpaper))
                    .w(px(PROMO_PIECE_PX))
                    .h(px(PROMO_PIECE_PX))
                    .object_fit(ObjectFit::Contain),
            )
            .on_click(cx.listener(move |this, ev, window, cx| {
                this.on_choose_promotion(top_promotes, ev, window, cx);
            }));

        let bottom_choice = div()
            .id("promote-choice-bottom")
            .w_full()
            .h(px(card_h / 2.0))
            .items_center()
            .justify_center()
            .border_t_1()
            .border_color(hsla(0.0, 0.0, 1.0, 0.1))
            .child(
                img(piece_asset_path(bottom_piece, self.piece_wallpaper))
                    .w(px(PROMO_PIECE_PX))
                    .h(px(PROMO_PIECE_PX))
                    .object_fit(ObjectFit::Contain),
            )
            .on_click(cx.listener(move |this, ev, window, cx| {
                this.on_choose_promotion(bottom_promotes, ev, window, cx);
            }));

        let card = div()
            .id("promotion-card")
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(card_w))
            .h(px(card_h))
            .rounded(px(PROMO_CARD_RADIUS))
            .border_1()
            .border_color(hsla(0.0, 0.0, 1.0, 0.16))
            .bg(hsla(0.0, 0.0, 0.08, 0.96))
            .overflow_hidden()
            .v_flex()
            .child(top_choice)
            .child(bottom_choice)
            .on_click(cx.listener(|_, _: &ClickEvent, _: &mut Window, cx| {
                cx.stop_propagation();
            }));

        div()
            .id("promotion-overlay")
            .absolute()
            .left_0()
            .top_0()
            .w(px(board_px))
            .h(px(board_px))
            .on_click(cx.listener(|this, ev, window, cx| {
                this.on_cancel_promotion(ev, window, cx);
            }))
            .child(card)
            .into_any_element()
    }

    fn render_shapes_overlay(&self, board_px: f32, scene_w: f32) -> AnyElement {
        #[derive(Clone)]
        struct ShapesPrepaint {
            shapes: Vec<DrawShape>,
            current: Option<DrawCurrent>,
            board_px: f32,
        }

        let bounds_cell = self.draw_scene_bounds.clone();
        let shapes = self.draw_shapes.clone();
        let current = self.draw_current;

        div()
            .id("shapes-overlay")
            .absolute()
            .left_0()
            .top_0()
            .w(px(scene_w))
            .h(px(board_px))
            .child(
                canvas(
                    move |bounds, _, _| {
                        *bounds_cell.borrow_mut() = Some(bounds);
                        ShapesPrepaint {
                            shapes: shapes.clone(),
                            current,
                            board_px,
                        }
                    },
                    move |bounds, prepaint, window, _| {
                        for shape in &prepaint.shapes {
                            GpuiP1Shell::paint_shape(
                                window,
                                *shape,
                                prepaint.board_px,
                                false,
                                bounds.origin,
                            );
                        }
                        if let Some(current) = prepaint.current {
                            GpuiP1Shell::paint_current_shape(
                                window,
                                current,
                                prepaint.board_px,
                                bounds.origin,
                            );
                        }
                    },
                )
                .size_full(),
            )
            .into_any_element()
    }

    fn render_drag_overlay(&self) -> AnyElement {
        let Some(drag) = self.drag else {
            return div().into_any_element();
        };
        if !drag.started {
            return div().into_any_element();
        }
        let Some(view_bounds) = *self.view_bounds.borrow() else {
            return div().into_any_element();
        };

        let piece = match drag.source {
            DragSource::Board { piece, .. } => piece,
            DragSource::Hand { piece_type, color } => Piece { piece_type, color },
        };
        let size = PIECE_PX;
        let local_cursor = point(
            drag.cursor.x - view_bounds.origin.x,
            drag.cursor.y - view_bounds.origin.y,
        );

        div()
            .absolute()
            .left(local_cursor.x - px(size * 0.5))
            .top(local_cursor.y - px(size * 0.5))
            .w(px(size))
            .h(px(size))
            .child(
                img(piece_asset_path(piece, self.piece_wallpaper))
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Contain)
                    .opacity(0.94),
            )
            .into_any_element()
    }

    fn render_hand_piece(
        &self,
        color: Color,
        piece_type: PieceType,
        slot_h: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let count = self.game.hand_count(color, piece_type);
        let dragging_same_hand_piece = matches!(
            self.drag,
            Some(DragState {
                started: true,
                source: DragSource::Hand {
                    color: drag_color,
                    piece_type: drag_piece_type,
                },
                ..
            }) if drag_color == color && drag_piece_type == piece_type
        );
        let effective_count = if dragging_same_hand_piece {
            count.saturating_sub(1)
        } else {
            count
        };
        let selected = self.game.selected_hand_piece() == Some(piece_type);
        let clickable = color == self.game.side_to_move() && count > 0;
        let dim = effective_count == 0;
        let bg = if selected {
            hsla(0.09, 0.78, 0.56, 0.22)
        } else {
            hsla(0.0, 0.0, 0.0, 0.0)
        };

        let piece = Piece { piece_type, color };
        let mut cell = div()
            .id(("hand-piece", color as usize * 16 + piece_type as usize))
            .relative()
            .w(px(HAND_COL_W))
            .h(px(slot_h))
            .flex_shrink_0()
            .bg(bg)
            .items_center()
            .justify_center()
            .child(
                img(piece_asset_path(piece, self.piece_wallpaper))
                    .w(px(HAND_PIECE_PX))
                    .h(px(HAND_PIECE_PX))
                    .object_fit(ObjectFit::Contain)
                    .opacity(if dragging_same_hand_piece {
                        0.06
                    } else if dim {
                        0.22
                    } else {
                        1.0
                    }),
            );
        if effective_count > 0 {
            cell = cell.child(
                div()
                    .absolute()
                    .right(px(2.0))
                    .top(px(3.0))
                    .min_w(px(14.0))
                    .px_1()
                    .rounded(px(6.0))
                    .bg(hsla(0.0, 0.0, 0.12, 0.85))
                    .text_size(px(11.0))
                    .text_color(gpui::white())
                    .child(effective_count.to_string()),
            );
        }

        cell = cell.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, ev, window, cx| {
                this.on_hand_mouse_down(color, piece_type, ev, window, cx);
            }),
        );
        cell = cell.on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, ev, window, cx| {
                this.on_hand_draw_start(color, piece_type, ev, window, cx);
            }),
        );

        if clickable {
            cell.on_click(cx.listener(move |this, ev, window, cx| {
                this.on_hand_piece_click(piece_type, ev, window, cx);
            }))
        } else {
            cell
        }
    }

    fn render_hand_panel(
        &self,
        color: Color,
        board_px: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let slot_h = board_px / HAND_PIECES.len() as f32;
        let mut col = v_flex()
            .h(px(board_px))
            .w(px(HAND_COL_W))
            .flex_shrink_0()
            .justify_between()
            .items_center()
            .p_1();
        for piece_type in HAND_PIECES {
            col = col.child(self.render_hand_piece(color, piece_type, slot_h, cx));
        }
        col
    }

    fn has_star_point_left_anchor(ui_row: u8, ui_col: u8) -> bool {
        // Right-side stars stay on the current cell top-left corner.
        matches!((ui_row, ui_col), (3, 6) | (6, 6))
    }

    fn has_star_point_right_anchor(ui_row: u8, ui_col: u8) -> bool {
        // Left-side stars are mirrored: anchor on current cell top-right corner,
        // which is visually equivalent to using the neighbor cell's top-left.
        matches!((ui_row, ui_col), (3, 2) | (6, 2))
    }

    fn render_top_file_coords(&self, board_px: f32) -> impl IntoElement {
        let mut row = h_flex()
            .absolute()
            .left(px(Self::board_left_x()))
            .top(px(-14.0))
            .w(px(board_px))
            .h(px(12.0))
            .gap_0()
            .items_center();
        for file in 0..BOARD_SIZE {
            row = row.child(
                div()
                    .w(px(CELL_PX))
                    .h_full()
                    .items_center()
                    .justify_center()
                    .text_size(px(11.0))
                    .text_color(hsla(0.0, 0.0, 0.76, 0.82))
                    .child((BOARD_SIZE - file).to_string()),
            );
        }
        row
    }

    fn render_right_rank_coords(&self, board_px: f32) -> impl IntoElement {
        let mut strip = div()
            .relative()
            .w(px(Self::board_to_right_hand_gap()))
            .h(px(board_px));
        for ui_row in 0..BOARD_SIZE {
            strip = strip.child(
                div()
                    .absolute()
                    .left(px(SCENE_GAP_PX))
                    .top(px((ui_row as f32 + 0.5) * CELL_PX - 7.0))
                    .w(px(BOARD_COORD_RIGHT_W))
                    .h(px(14.0))
                    .items_center()
                    .justify_center()
                    .text_size(px(11.0))
                    .text_color(hsla(0.0, 0.0, 0.76, 0.82))
                    .child((ui_row + 1).to_string()),
            );
        }
        strip
    }

    fn square_bg(&self, sq: Square) -> Hsla {
        if self.game.selected() == Some(sq) {
            return hsla(0.13, 0.64, 0.68, 0.32);
        }
        if self.game.is_legal_destination(sq) {
            return hsla(0.31, 0.50, 0.66, 0.26);
        }
        if self.game.last_action_from() == Some(sq) || self.game.last_action_to() == Some(sq) {
            return hsla(0.58, 0.78, 0.74, 0.28);
        }
        hsla(0.0, 0.0, 0.0, 0.0)
    }

    fn render_board_scene(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut board = v_flex().gap_0();
        for rank in 0..BOARD_SIZE {
            let mut row = h_flex().gap_0();
            for file in 0..BOARD_SIZE {
                let sq = Self::square_from_ui(rank, file);
                let piece = self.game.piece_at(sq);
                let mut cell = div()
                    .id(("sq", (rank as usize) * BOARD_SIZE as usize + file as usize))
                    .relative()
                    .w(px(CELL_PX))
                    .h(px(CELL_PX))
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_r(px(1.8))
                    .border_b(px(1.8))
                    .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                    .bg(self.square_bg(sq))
                    .text_color(cx.theme().foreground);
                if Self::has_star_point_left_anchor(rank, file) {
                    cell = cell.child(
                        div()
                            .absolute()
                            .left(px(-4.2))
                            .top(px(-4.2))
                            .w(px(8.4))
                            .h(px(8.4))
                            .rounded_full()
                            .bg(hsla(0.0, 0.0, 0.0, 1.0)),
                    );
                }
                if Self::has_star_point_right_anchor(rank, file) {
                    cell = cell.child(
                        div()
                            .absolute()
                            .left(px(CELL_PX - 7.2))
                            .top(px(-5.2))
                            .w(px(8.4))
                            .h(px(8.4))
                            .rounded_full()
                            .bg(hsla(0.0, 0.0, 0.0, 1.0)),
                    );
                }
                let dragging_from_sq = matches!(
                    self.drag,
                    Some(DragState {
                        started: true,
                        source: DragSource::Board { from, .. },
                        ..
                    }) if from == sq
                );
                if let Some(p) = piece.filter(|_| !dragging_from_sq) {
                    cell = cell.child(
                        img(piece_asset_path(p, self.piece_wallpaper))
                            .w(px(PIECE_PX))
                            .h(px(PIECE_PX))
                            .object_fit(ObjectFit::Contain),
                    );
                }
                row = row.child(
                    cell.on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev, window, cx| {
                            this.on_board_mouse_down(sq, ev, window, cx);
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, ev, window, cx| {
                            this.on_board_draw_start(sq, ev, window, cx);
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, ev, window, cx| {
                            this.on_board_mouse_up(sq, ev, window, cx);
                        }),
                    )
                    .on_click(cx.listener(move |this, ev, window, cx| {
                        this.on_square_click(sq, ev, window, cx);
                    })),
                );
            }
            board = board.child(row);
        }

        let board_px = CELL_PX * BOARD_SIZE as f32;
        let scene_w = board_px + HAND_COL_W * 2.0 + SCENE_GAP_PX + Self::board_to_right_hand_gap();
        let board_panel = div()
            .relative()
            .w(px(board_px))
            .h(px(board_px))
            .flex_shrink_0()
            .overflow_hidden()
            .child(
                img(board_asset_path(self.board_wallpaper))
                    .absolute()
                    .left_0()
                    .top_0()
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Fill),
            )
            .child(div().relative().w_full().h_full().child(board))
            .child(self.render_promotion_overlay(board_px, cx));

        div()
            .id("board-scene")
            .relative()
            .w(px(scene_w))
            .h(px(board_px))
            .flex_shrink_0()
            .child(
                h_flex()
                    .gap_0()
                    .items_start()
                    .child(self.render_hand_panel(Color::White, board_px, cx))
                    .child(div().w(px(SCENE_GAP_PX)).h(px(board_px)))
                    .child(board_panel)
                    .child(self.render_right_rank_coords(board_px))
                    .child(self.render_hand_panel(Color::Black, board_px, cx)),
            )
            .child(self.render_top_file_coords(board_px))
            .child(self.render_shapes_overlay(board_px, scene_w))
            .into_any_element()
    }

    fn render_center_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let status_line = if self.game.status().is_empty() {
            "Status: ready".to_string()
        } else {
            format!("Status: {}", self.game.status())
        };
        let selected_hand_line = match self.game.selected_hand_piece() {
            Some(piece_type) => format!("Selected hand: {}", piece_type_label(piece_type)),
            None => "Selected hand: none".to_string(),
        };

        v_flex()
            .size_full()
            .items_center()
            .justify_start()
            .gap_2()
            .p_3()
            .child(
                h_flex()
                    .w_full()
                    .items_start()
                    .justify_between()
                    .text_size(px(13.0))
                    .text_color(hsla(0.0, 0.0, 0.86, 0.9))
                    .child(selected_hand_line)
                    .child(
                        h_flex()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .pr_5()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();
                            })
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(status_line),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_1()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .bg(cx.theme().secondary)
                                    .child(
                                        Button::new("clear-drawings")
                                            .ghost()
                                            .small()
                                            .icon(Icon::new(IconName::Delete).size_4())
                                            .tooltip("Clear right-click markup")
                                            .on_click(cx.listener(|this, ev, window, cx| {
                                                this.on_clear_drawings(ev, window, cx);
                                            })),
                                    ),
                            ),
                    ),
            )
            .child(
                div().w_full().flex_1().child(
                    h_flex()
                        .w_full()
                        .justify_center()
                        .child(self.render_board_scene(cx)),
                ),
            )
            .into_any_element()
    }

    pub(crate) fn render_right_sidebar(&self, cx: &App) -> AnyElement {
        let turn = format!("Turn: {:?}", self.game.side_to_move());
        let ply = format!("Ply: {}", self.game.ply());
        let status_line = if self.game.status().is_empty() {
            "Status: ready".to_string()
        } else {
            format!("Status: {}", self.game.status())
        };

        v_flex()
            .size_full()
            .gap_3()
            .p_3()
            .border_l_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child("Inspector")
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("right panel"),
                    ),
            )
            .child(turn)
            .child(ply)
            .child(status_line)
            .child(
                div()
                    .mt_2()
                    .border_t_1()
                    .border_color(cx.theme().sidebar_border),
            )
            .child("Annotations")
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(cx.theme().muted_foreground)
                    .child("Right click on board to draw circles/arrows."),
            )
            .into_any_element()
    }

    pub(crate) fn render_bottom_panel(&self, cx: &App) -> AnyElement {
        let status_line = if self.game.status().is_empty() {
            "ready".to_string()
        } else {
            self.game.status().to_string()
        };

        v_flex()
            .size_full()
            .gap_2()
            .p_2()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(Button::new("bottom-tab-console").ghost().label("Console"))
                    .child(Button::new("bottom-tab-analysis").ghost().label("Analysis"))
                    .child(Button::new("bottom-tab-moves").ghost().label("Moves")),
            )
            .child(div().h(px(1.0)).w_full().bg(cx.theme().border))
            .child(format!("engine: idle | game status: {status_line}"))
            .child("workspace initialized: dock layout active")
            .into_any_element()
    }
}

impl Render for GpuiP1Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Trigger all board/piece image decoders early, so first visible frame is stable.
        let mut preload = div()
            .absolute()
            .left(px(-10_000.0))
            .top(px(-10_000.0))
            .w(px(1.0))
            .h(px(1.0));
        preload = preload.child(
            img(board_asset_path(self.board_wallpaper))
                .w(px(1.0))
                .h(px(1.0))
                .object_fit(ObjectFit::Fill),
        );
        for color in [Color::Black, Color::White] {
            for piece_type in HAND_PIECES {
                preload = preload.child(
                    img(piece_asset_path(Piece { piece_type, color }, self.piece_wallpaper))
                        .w(px(1.0))
                        .h(px(1.0))
                        .object_fit(ObjectFit::Contain),
                );
            }
        }

        let main_content = div().size_full().child(self.render_center_panel(cx));
        let view_bounds = self.view_bounds.clone();
        let bounds_tracker = canvas(
            move |bounds, _, _| {
                *view_bounds.borrow_mut() = Some(bounds);
            },
            |_, _, _, _| {},
        );

        let content_layer = div()
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(bounds_tracker.absolute().left_0().top_0().w_full().h_full())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev, window, cx| {
                    this.on_root_mouse_down(ev, window, cx);
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, ev, window, cx| {
                    this.on_root_mouse_down(ev, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, ev, window, cx| {
                this.on_root_mouse_move(ev, window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, ev, window, cx| {
                    this.on_root_mouse_up(ev, window, cx);
                }),
            )
            .on_mouse_up(
                MouseButton::Right,
                cx.listener(|this, ev, window, cx| {
                    this.on_root_right_mouse_up(ev, window, cx);
                }),
            )
            .child(preload)
            .child(main_content);

        div()
            .relative()
            .size_full()
            .child(content_layer)
            .child(self.render_drag_overlay())
    }
}
