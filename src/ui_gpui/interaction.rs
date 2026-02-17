use gpui::*;
use shogi::{Color, PieceType, Square};

use super::model::{
    BOARD_SIZE, CELL_PX, DRAG_START_THRESHOLD_PX, DragSource, DragState, DrawAnchor, DrawCurrent,
    GpuiP1Shell,
};

impl GpuiP1Shell {
    fn clear_drawings_if_outside_scene(&mut self, position: Point<Pixels>) -> bool {
        let Some(bounds) = *self.draw_scene_bounds.borrow() else {
            return false;
        };
        let inside_scene = position.x >= bounds.left()
            && position.x <= bounds.right()
            && position.y >= bounds.top()
            && position.y <= bounds.bottom();
        if inside_scene {
            return false;
        }

        let had_any = self.draw_current.is_some() || !self.draw_shapes.is_empty();
        if had_any {
            self.draw_current = None;
            self.draw_shapes.clear();
        }
        had_any
    }

    pub(crate) fn on_root_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left && event.button != MouseButton::Right {
            return;
        }
        if self.clear_drawings_if_outside_scene(event.position) {
            cx.notify();
        }
    }

    pub(crate) fn on_clear_drawings(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.draw_current = None;
        if !self.draw_shapes.is_empty() {
            self.draw_shapes.clear();
        }
        cx.notify();
    }

    pub(crate) fn on_square_click(
        &mut self,
        sq: Square,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.consume_suppressed_click() {
            return;
        }
        self.game.on_square_clicked(sq);
        self.play_pending_sound();
        cx.notify();
    }

    pub(crate) fn on_hand_piece_click(
        &mut self,
        piece_type: PieceType,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.consume_suppressed_click() {
            return;
        }
        self.game.select_hand_piece(piece_type);
        self.play_pending_sound();
        cx.notify();
    }

    pub(crate) fn on_choose_promotion(
        &mut self,
        promote: bool,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.game.choose_promotion(promote);
        self.play_pending_sound();
        cx.notify();
    }

    pub(crate) fn on_cancel_promotion(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.game.cancel_promotion();
        cx.notify();
    }

    pub(crate) fn on_board_mouse_down(
        &mut self,
        sq: Square,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cancel_draw_if_any();
        if event.button != MouseButton::Left || self.game.has_pending_promotion() {
            return;
        }
        let Some(piece) = self.game.piece_at(sq) else {
            return;
        };
        if piece.color != self.game.side_to_move() {
            return;
        }
        self.drag = Some(DragState {
            source: DragSource::Board { from: sq, piece },
            origin: event.position,
            cursor: event.position,
            started: false,
        });
        cx.notify();
    }

    pub(crate) fn on_board_draw_start(
        &mut self,
        sq: Square,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.game.has_pending_promotion() {
            return;
        }

        self.drag = None;
        self.game.clear_active_selection();
        let brush = Self::draw_brush(event.modifiers);
        let board_px = CELL_PX * BOARD_SIZE as f32;
        let anchor = DrawAnchor::Board(sq);
        let anchor_local = Self::anchor_to_scene_point(anchor, board_px);
        self.draw_current = Some(DrawCurrent {
            orig: anchor,
            dest: Some(anchor),
            cursor: anchor_local,
            brush,
        });
        cx.notify();
    }

    pub(crate) fn on_hand_mouse_down(
        &mut self,
        color: Color,
        piece_type: PieceType,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cancel_draw_if_any();
        if event.button != MouseButton::Left || self.game.has_pending_promotion() {
            return;
        }
        if color != self.game.side_to_move() || self.game.hand_count(color, piece_type) == 0 {
            return;
        }
        self.drag = Some(DragState {
            source: DragSource::Hand { piece_type, color },
            origin: event.position,
            cursor: event.position,
            started: false,
        });
        cx.notify();
    }

    pub(crate) fn on_hand_draw_start(
        &mut self,
        color: Color,
        piece_type: PieceType,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.game.has_pending_promotion() {
            return;
        }

        self.drag = None;
        self.game.clear_active_selection();
        let brush = Self::draw_brush(event.modifiers);
        let anchor = DrawAnchor::Hand { color, piece_type };
        let board_px = CELL_PX * BOARD_SIZE as f32;
        let anchor_local = Self::anchor_to_scene_point(anchor, board_px);
        self.draw_current = Some(DrawCurrent {
            orig: anchor,
            dest: Some(anchor),
            cursor: anchor_local,
            brush,
        });
        cx.notify();
    }

    pub(crate) fn on_board_mouse_up(
        &mut self,
        sq: Square,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        if !drag.started {
            self.drag = Some(drag);
            return;
        }

        match drag.source {
            DragSource::Board { from, .. } => self.game.perform_board_drag(from, sq),
            DragSource::Hand { piece_type, .. } => self.game.perform_hand_drag(piece_type, sq),
        }
        self.play_pending_sound();
        self.suppress_next_click = true;
        cx.stop_propagation();
        cx.notify();
    }

    pub(crate) fn on_root_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(mut draw) = self.draw_current {
            // GPUI reports window coordinates; convert to board scene local coordinates.
            if let Some(local) = self.scene_local_from_window(event.position) {
                draw.cursor = local;
                draw.dest = self.anchor_from_scene_local(local);
                self.draw_current = Some(draw);
                cx.notify();
            }
            return;
        }

        let Some(mut drag) = self.drag else {
            return;
        };
        drag.cursor = event.position;
        if !drag.started && self.drag_distance_px(drag) >= DRAG_START_THRESHOLD_PX {
            drag.started = true;
            match drag.source {
                DragSource::Board { from, .. } => self.game.preview_board_drag_from(from),
                DragSource::Hand { piece_type, .. } => self.game.preview_hand_drag_from(piece_type),
            }
        }
        self.drag = Some(drag);
        cx.notify();
    }

    pub(crate) fn on_root_mouse_up(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        if drag.started {
            self.game.clear_active_selection();
            self.suppress_next_click = true;
            cx.stop_propagation();
            cx.notify();
        }
    }

    pub(crate) fn on_root_right_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(mut draw) = self.draw_current.take() else {
            return;
        };

        if draw.dest.is_none() {
            draw.dest = self.anchor_from_window(event.position).or(Some(draw.orig));
        }
        if let Some(dest) = draw.dest {
            self.toggle_draw_shape(draw.orig, dest, draw.brush);
        }
        cx.stop_propagation();
        cx.notify();
    }
}
