use gpui::*;
use shogi::{Color, Square};
use std::f32::consts::TAU;

use super::model::{
    BOARD_COORD_RIGHT_W, BOARD_SIZE, CELL_PX, DrawAnchor, DrawBrush, DrawCurrent, DrawShape,
    GpuiP1Shell, HAND_COL_W, HAND_PIECES, SCENE_GAP_PX,
};

impl GpuiP1Shell {
    pub(crate) fn draw_brush(mods: Modifiers) -> DrawBrush {
        let mod_a = mods.shift || mods.control;
        let mod_b = mods.alt || mods.platform;
        match (mod_a, mod_b) {
            (false, false) => DrawBrush::Primary,
            (true, false) => DrawBrush::Alternative0,
            (false, true) => DrawBrush::Alternative1,
            (true, true) => DrawBrush::Alternative2,
        }
    }

    pub(crate) fn draw_brush_color(brush: DrawBrush, alpha: f32) -> Hsla {
        match brush {
            DrawBrush::Primary => hsla(0.34, 0.70, 0.28, alpha),
            DrawBrush::Alternative0 => hsla(0.00, 0.62, 0.33, alpha),
            DrawBrush::Alternative1 => hsla(0.61, 1.00, 0.27, alpha),
            DrawBrush::Alternative2 => hsla(0.10, 1.00, 0.45, alpha),
        }
    }

    pub(crate) fn toggle_draw_shape(
        &mut self,
        orig: DrawAnchor,
        dest: DrawAnchor,
        brush: DrawBrush,
    ) {
        if let Some(index) = self
            .draw_shapes
            .iter()
            .position(|shape| shape.orig == orig && shape.dest == dest)
        {
            if self.draw_shapes[index].brush == brush {
                self.draw_shapes.remove(index);
                return;
            }
            self.draw_shapes.remove(index);
        }

        self.draw_shapes.push(DrawShape { orig, dest, brush });
    }

    pub(crate) fn square_from_ui(ui_row: u8, ui_col: u8) -> Square {
        // Match shogiground/egui sente view: files are rendered 9 -> 1 from left to right.
        let file = BOARD_SIZE - 1 - ui_col;
        let rank = ui_row;
        Square::new(file, rank).expect("valid board coordinate")
    }

    pub(crate) fn ui_pos_from_square(sq: Square) -> (u8, u8) {
        let row = sq.rank();
        let col = BOARD_SIZE - 1 - sq.file();
        (row, col)
    }

    pub(crate) fn scene_slot_h(board_px: f32) -> f32 {
        board_px / HAND_PIECES.len() as f32
    }

    pub(crate) fn board_left_x() -> f32 {
        HAND_COL_W + SCENE_GAP_PX
    }

    pub(crate) fn right_hand_left_x(board_px: f32) -> f32 {
        Self::board_left_x() + board_px + Self::board_to_right_hand_gap()
    }

    pub(crate) fn board_to_right_hand_gap() -> f32 {
        SCENE_GAP_PX + BOARD_COORD_RIGHT_W
    }

    pub(crate) fn anchor_to_scene_point(anchor: DrawAnchor, board_px: f32) -> Point<Pixels> {
        match anchor {
            DrawAnchor::Board(sq) => {
                let (ui_row, ui_col) = Self::ui_pos_from_square(sq);
                point(
                    px(Self::board_left_x() + (ui_col as f32 + 0.5) * CELL_PX),
                    px((ui_row as f32 + 0.5) * CELL_PX),
                )
            }
            DrawAnchor::Hand { color, piece_type } => {
                let slot_h = Self::scene_slot_h(board_px);
                let idx = HAND_PIECES
                    .iter()
                    .position(|&pt| pt == piece_type)
                    .unwrap_or(HAND_PIECES.len() - 1);
                let x = if color == Color::White {
                    HAND_COL_W * 0.5
                } else {
                    Self::right_hand_left_x(board_px) + HAND_COL_W * 0.5
                };
                let y = (idx as f32 + 0.5) * slot_h;
                point(px(x), px(y))
            }
        }
    }

    pub(crate) fn anchor_from_scene_local(&self, local: Point<Pixels>) -> Option<DrawAnchor> {
        let board_px = CELL_PX * BOARD_SIZE as f32;
        let x = local.x / px(1.0);
        let y = local.y / px(1.0);

        if (0.0..board_px).contains(&y) {
            if (0.0..HAND_COL_W).contains(&x) {
                let idx = ((y / Self::scene_slot_h(board_px)).floor() as usize)
                    .min(HAND_PIECES.len() - 1);
                return Some(DrawAnchor::Hand {
                    color: Color::White,
                    piece_type: HAND_PIECES[idx],
                });
            }

            let board_left = Self::board_left_x();
            if (board_left..(board_left + board_px)).contains(&x) {
                let ui_col = ((x - board_left) / CELL_PX).floor() as u8;
                let ui_row = (y / CELL_PX).floor() as u8;
                return Some(DrawAnchor::Board(Self::square_from_ui(ui_row, ui_col)));
            }

            let right_left = Self::right_hand_left_x(board_px);
            if (right_left..(right_left + HAND_COL_W)).contains(&x) {
                let idx = ((y / Self::scene_slot_h(board_px)).floor() as usize)
                    .min(HAND_PIECES.len() - 1);
                return Some(DrawAnchor::Hand {
                    color: Color::Black,
                    piece_type: HAND_PIECES[idx],
                });
            }
        }

        None
    }

    pub(crate) fn scene_local_from_window(&self, pos: Point<Pixels>) -> Option<Point<Pixels>> {
        let bounds = (*self.draw_scene_bounds.borrow())?;
        Some(point(pos.x - bounds.origin.x, pos.y - bounds.origin.y))
    }

    pub(crate) fn anchor_from_window(&self, pos: Point<Pixels>) -> Option<DrawAnchor> {
        let local = self.scene_local_from_window(pos)?;
        self.anchor_from_scene_local(local)
    }

    pub(crate) fn paint_shape(
        window: &mut Window,
        shape: DrawShape,
        board_px: f32,
        current: bool,
        canvas_origin: Point<Pixels>,
    ) {
        let orig_local = Self::anchor_to_scene_point(shape.orig, board_px);
        let dest_local = Self::anchor_to_scene_point(shape.dest, board_px);
        let orig = point(
            orig_local.x + canvas_origin.x,
            orig_local.y + canvas_origin.y,
        );
        let dest = point(
            dest_local.x + canvas_origin.x,
            dest_local.y + canvas_origin.y,
        );
        let color = Self::draw_brush_color(shape.brush, if current { 0.62 } else { 0.82 });
        if shape.orig == shape.dest {
            Self::paint_circle(window, orig, color);
        } else {
            Self::paint_arrow(window, orig, dest, color);
        }
    }

    pub(crate) fn paint_current_shape(
        window: &mut Window,
        current: DrawCurrent,
        board_px: f32,
        canvas_origin: Point<Pixels>,
    ) {
        let orig_local = Self::anchor_to_scene_point(current.orig, board_px);
        let orig = point(
            orig_local.x + canvas_origin.x,
            orig_local.y + canvas_origin.y,
        );
        let color = Self::draw_brush_color(current.brush, 0.62);
        if let Some(dest_anchor) = current.dest {
            let dest_local = Self::anchor_to_scene_point(dest_anchor, board_px);
            let dest = point(
                dest_local.x + canvas_origin.x,
                dest_local.y + canvas_origin.y,
            );
            if dest_anchor == current.orig {
                Self::paint_circle(window, orig, color);
            } else {
                Self::paint_arrow(window, orig, dest, color);
            }
        } else {
            let cursor = point(
                current.cursor.x + canvas_origin.x,
                current.cursor.y + canvas_origin.y,
            );
            Self::paint_arrow(window, orig, cursor, color);
        }
    }

    pub(crate) fn paint_circle(window: &mut Window, center: Point<Pixels>, color: Hsla) {
        let radius = CELL_PX * 0.34;
        let stroke = CELL_PX * 0.06;
        let mut builder = PathBuilder::stroke(px(stroke));

        let segments = 40usize;
        for i in 0..=segments {
            let t = TAU * (i as f32 / segments as f32);
            let x = center.x + px(radius * t.cos());
            let y = center.y + px(radius * t.sin());
            if i == 0 {
                builder.move_to(point(x, y));
            } else {
                builder.line_to(point(x, y));
            }
        }
        builder.close();

        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }

    pub(crate) fn paint_arrow(
        window: &mut Window,
        start: Point<Pixels>,
        end: Point<Pixels>,
        color: Hsla,
    ) {
        let sx = start.x / px(1.0);
        let sy = start.y / px(1.0);
        let ex = end.x / px(1.0);
        let ey = end.y / px(1.0);
        let dx = ex - sx;
        let dy = ey - sy;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1.0 {
            return;
        }

        let ux = dx / len;
        let uy = dy / len;
        let margin = CELL_PX * 0.16;
        let line_end_x = ex - ux * margin;
        let line_end_y = ey - uy * margin;

        let mut line = PathBuilder::stroke(px(CELL_PX * 0.10));
        line.move_to(point(px(sx), px(sy)));
        line.line_to(point(px(line_end_x), px(line_end_y)));
        if let Ok(path) = line.build() {
            window.paint_path(path, color);
        }

        let head_len = CELL_PX * 0.32;
        let head_half_w = CELL_PX * 0.16;
        let base_x = line_end_x - ux * head_len;
        let base_y = line_end_y - uy * head_len;
        let perp_x = -uy;
        let perp_y = ux;

        let mut head = PathBuilder::fill();
        head.move_to(point(px(ex), px(ey)));
        head.line_to(point(
            px(base_x + perp_x * head_half_w),
            px(base_y + perp_y * head_half_w),
        ));
        head.line_to(point(
            px(base_x - perp_x * head_half_w),
            px(base_y - perp_y * head_half_w),
        ));
        head.close();
        if let Ok(path) = head.build() {
            window.paint_path(path, color);
        }
    }
}
