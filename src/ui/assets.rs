use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use shogi::{Color, Piece, PieceType};

pub const PIECE_DRAW_SIZE: u32 = 52;

#[derive(Default)]
pub struct UiAssets {
    pub board_texture: Option<TextureHandle>,
    pub piece_textures: HashMap<String, TextureHandle>,
}

pub fn load_assets(ctx: &Context) -> Result<UiAssets, String> {
    let mut assets = UiAssets::default();

    let board_path = Path::new("assets/boards/lishogi/kaya1.jpg");
    assets.board_texture = Some(load_board_texture(ctx, board_path)?);

    let pieces_dir = Path::new("assets/pieces/standard/western");
    for color in [Color::Black, Color::White] {
        for piece_type in [
            PieceType::King,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Gold,
            PieceType::Silver,
            PieceType::Knight,
            PieceType::Lance,
            PieceType::Pawn,
            PieceType::ProRook,
            PieceType::ProBishop,
            PieceType::ProSilver,
            PieceType::ProKnight,
            PieceType::ProLance,
            PieceType::ProPawn,
        ] {
            let key = piece_asset_key(Piece { piece_type, color });
            let path = pieces_dir.join(format!("{key}.svg"));
            let texture = load_svg_texture(ctx, &path, PIECE_DRAW_SIZE, &format!("piece-{key}"))
                .map_err(|e| format!("failed to load piece {key}: {e}"))?;
            assets.piece_textures.insert(key, texture);
        }
    }

    Ok(assets)
}

pub fn piece_asset_key(piece: Piece) -> String {
    let prefix = if piece.color == Color::Black {
        "0"
    } else {
        "1"
    };
    let code = match piece.piece_type {
        PieceType::Pawn => "FU",
        PieceType::Lance => "KY",
        PieceType::Knight => "KE",
        PieceType::Silver => "GI",
        PieceType::Gold => "KI",
        PieceType::Bishop => "KA",
        PieceType::Rook => "HI",
        PieceType::King => {
            if piece.color == Color::Black {
                "OU"
            } else {
                "GY"
            }
        }
        PieceType::ProPawn => "TO",
        PieceType::ProLance => "NY",
        PieceType::ProKnight => "NK",
        PieceType::ProSilver => "NG",
        PieceType::ProBishop => "UM",
        PieceType::ProRook => "RY",
    };
    format!("{prefix}{code}")
}

fn load_board_texture(ctx: &Context, path: &Path) -> Result<TextureHandle, String> {
    let dyn_img = image::open(path).map_err(|e| format!("cannot open board {:?}: {e}", path))?;
    let rgba = dyn_img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
    Ok(ctx.load_texture("board-kaya1", image, TextureOptions::LINEAR))
}

fn load_svg_texture(
    ctx: &Context,
    path: &Path,
    target_px: u32,
    texture_name: &str,
) -> Result<TextureHandle, String> {
    let svg_bytes = std::fs::read(path).map_err(|e| format!("cannot read {:?}: {e}", path))?;

    let mut options = usvg::Options {
        resources_dir: path.parent().map(PathBuf::from),
        ..usvg::Options::default()
    };
    options.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(&svg_bytes, &options)
        .map_err(|e| format!("cannot parse svg {:?}: {e}", path))?;

    let mut pixmap = tiny_skia::Pixmap::new(target_px, target_px)
        .ok_or_else(|| "cannot create pixmap".to_string())?;
    let sx = target_px as f32 / tree.size().width();
    let sy = target_px as f32 / tree.size().height();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );

    let image = ColorImage::from_rgba_unmultiplied(
        [pixmap.width() as usize, pixmap.height() as usize],
        pixmap.data(),
    );
    Ok(ctx.load_texture(texture_name, image, TextureOptions::LINEAR))
}
