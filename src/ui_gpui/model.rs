use crate::core::game::GameState;
use gpui::*;
use shogi::{Color, Piece, PieceType, Square};
use std::{cell::RefCell, rc::Rc};

use super::assets::{BoardWallpaper, PieceWallpaper};
use super::sound::SoundPlayer;

pub(crate) const BOARD_SIZE: u8 = 9;
// Use integer pixel sizes to avoid sub-pixel misalignment between cells/highlights/grid lines.
pub(crate) const CELL_PX: f32 = 73.0;
pub(crate) const PIECE_PX: f32 = 65.0;
pub(crate) const HAND_COL_W: f32 = 96.0;
pub(crate) const HAND_PIECE_PX: f32 = 73.0;
pub(crate) const PROMO_CARD_W_RATIO: f32 = 0.9;
pub(crate) const PROMO_CARD_H_RATIO: f32 = 2.0;
pub(crate) const PROMO_CARD_RADIUS: f32 = 12.0;
pub(crate) const PROMO_PIECE_PX: f32 = 58.0;
pub(crate) const DRAG_START_THRESHOLD_PX: f32 = 4.0;
pub(crate) const SCENE_GAP_PX: f32 = 8.0;
pub(crate) const BOARD_COORD_RIGHT_W: f32 = 16.0;
pub(crate) const HAND_PIECES: [PieceType; 7] = [
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Gold,
    PieceType::Silver,
    PieceType::Knight,
    PieceType::Lance,
    PieceType::Pawn,
];

pub(crate) struct GpuiP1Shell {
    pub(crate) game: GameState,
    pub(crate) drag: Option<DragState>,
    pub(crate) draw_current: Option<DrawCurrent>,
    pub(crate) draw_shapes: Vec<DrawShape>,
    pub(crate) draw_scene_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    pub(crate) view_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    pub(crate) suppress_next_click: bool,
    pub(crate) sound: SoundPlayer,
    pub(crate) piece_wallpaper: PieceWallpaper,
    pub(crate) board_wallpaper: BoardWallpaper,
}

#[derive(Clone, Copy)]
pub(crate) enum DragSource {
    Board { from: Square, piece: Piece },
    Hand { piece_type: PieceType, color: Color },
}

#[derive(Clone, Copy)]
pub(crate) struct DragState {
    pub(crate) source: DragSource,
    pub(crate) origin: Point<Pixels>,
    pub(crate) cursor: Point<Pixels>,
    pub(crate) started: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrawAnchor {
    Board(Square),
    Hand { color: Color, piece_type: PieceType },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrawBrush {
    Primary,
    Alternative0,
    Alternative1,
    Alternative2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrawShape {
    pub(crate) orig: DrawAnchor,
    pub(crate) dest: DrawAnchor,
    pub(crate) brush: DrawBrush,
}

#[derive(Clone, Copy)]
pub(crate) struct DrawCurrent {
    pub(crate) orig: DrawAnchor,
    pub(crate) dest: Option<DrawAnchor>,
    pub(crate) cursor: Point<Pixels>,
    pub(crate) brush: DrawBrush,
}

impl GpuiP1Shell {
    pub(crate) fn new() -> Self {
        Self {
            game: GameState::new(),
            drag: None,
            draw_current: None,
            draw_shapes: Vec::new(),
            draw_scene_bounds: Rc::new(RefCell::new(None)),
            view_bounds: Rc::new(RefCell::new(None)),
            suppress_next_click: false,
            sound: SoundPlayer::new(),
            piece_wallpaper: PieceWallpaper::RyokoKanji,
            board_wallpaper: BoardWallpaper::Oak,
        }
    }

    pub(crate) fn cancel_draw_if_any(&mut self) {
        self.draw_current = None;
    }

    pub(crate) fn drag_distance_px(&self, drag: DragState) -> f32 {
        let dx = (drag.cursor.x - drag.origin.x).abs() / px(1.0);
        let dy = (drag.cursor.y - drag.origin.y).abs() / px(1.0);
        (dx * dx + dy * dy).sqrt()
    }

    pub(crate) fn consume_suppressed_click(&mut self) -> bool {
        if self.suppress_next_click {
            self.suppress_next_click = false;
            true
        } else {
            false
        }
    }

    pub(crate) fn play_pending_sound(&mut self) {
        if let Some(cue) = self.game.take_pending_sound() {
            self.sound.play(cue);
        }
    }

    pub(crate) fn piece_wallpaper(&self) -> PieceWallpaper {
        self.piece_wallpaper
    }

    pub(crate) fn board_wallpaper(&self) -> BoardWallpaper {
        self.board_wallpaper
    }

    pub(crate) fn set_piece_wallpaper(&mut self, wallpaper: PieceWallpaper) {
        self.piece_wallpaper = wallpaper;
    }

    pub(crate) fn set_board_wallpaper(&mut self, wallpaper: BoardWallpaper) {
        self.board_wallpaper = wallpaper;
    }
}
