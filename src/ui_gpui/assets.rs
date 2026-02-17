use std::borrow::Cow;

use anyhow::{Result, anyhow};
use gpui::{AssetSource, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "boards/**/*.jpg"]
#[include = "boards/**/*.png"]
#[include = "pieces/standard/ryoko_1kanji/**/*.svg"]
#[include = "pieces/standard/western/**/*.svg"]
struct ProjectAssets;

pub struct CombinedAssets;

impl AssetSource for CombinedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        if let Some(file) = ProjectAssets::get(path) {
            return Ok(Some(file.data));
        }

        let fallback = gpui_component_assets::Assets;
        fallback
            .load(path)
            .or_else(|_| Err(anyhow!("could not find asset at path \"{path}\"")))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut out: Vec<SharedString> = ProjectAssets::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect();

        let fallback = gpui_component_assets::Assets;
        if let Ok(mut extra) = fallback.list(path) {
            out.append(&mut extra);
        }

        out.sort_unstable();
        out.dedup();
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceWallpaper {
    RyokoKanji,
    Western,
}

impl PieceWallpaper {
    pub fn all() -> [Self; 2] {
        [Self::RyokoKanji, Self::Western]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RyokoKanji => "Ryoko 1kanji",
            Self::Western => "Western",
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Self::RyokoKanji => "ryoko_1kanji",
            Self::Western => "western",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardWallpaper {
    Oak,
    Kaya1,
    Kaya2,
    Wood,
    Wood1,
    Kinkaku,
    Painting1,
    Painting2,
    Space,
    Dobutsu,
    DobutsuFlip,
}

impl BoardWallpaper {
    pub fn all() -> [Self; 11] {
        [
            Self::Oak,
            Self::Kaya1,
            Self::Kaya2,
            Self::Wood,
            Self::Wood1,
            Self::Kinkaku,
            Self::Painting1,
            Self::Painting2,
            Self::Space,
            Self::Dobutsu,
            Self::DobutsuFlip,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Oak => "Oak",
            Self::Kaya1 => "Kaya 1",
            Self::Kaya2 => "Kaya 2",
            Self::Wood => "Wood",
            Self::Wood1 => "Wood 1",
            Self::Kinkaku => "Kinkaku",
            Self::Painting1 => "Painting 1",
            Self::Painting2 => "Painting 2",
            Self::Space => "Space",
            Self::Dobutsu => "Dobutsu",
            Self::DobutsuFlip => "Dobutsu Flip",
        }
    }
}

pub fn board_asset_path(board: BoardWallpaper) -> &'static str {
    match board {
        BoardWallpaper::Oak => "boards/lishogi/oak.png",
        BoardWallpaper::Kaya1 => "boards/lishogi/kaya1.jpg",
        BoardWallpaper::Kaya2 => "boards/lishogi/kaya2.jpg",
        BoardWallpaper::Wood => "boards/lishogi/wood.png",
        BoardWallpaper::Wood1 => "boards/lishogi/wood1.jpg",
        BoardWallpaper::Kinkaku => "boards/lishogi/kinkaku.jpg",
        BoardWallpaper::Painting1 => "boards/lishogi/painting1.jpg",
        BoardWallpaper::Painting2 => "boards/lishogi/painting2.jpg",
        BoardWallpaper::Space => "boards/lishogi/space.png",
        BoardWallpaper::Dobutsu => "boards/lishogi/dobutsu.png",
        BoardWallpaper::DobutsuFlip => "boards/lishogi/dobutsu_flip.png",
    }
}

pub fn piece_asset_path(piece: shogi::Piece, wallpaper: PieceWallpaper) -> String {
    let prefix = if piece.color == shogi::Color::Black {
        "0"
    } else {
        "1"
    };
    let code = match piece.piece_type {
        shogi::PieceType::Pawn => "FU",
        shogi::PieceType::Lance => "KY",
        shogi::PieceType::Knight => "KE",
        shogi::PieceType::Silver => "GI",
        shogi::PieceType::Gold => "KI",
        shogi::PieceType::Bishop => "KA",
        shogi::PieceType::Rook => "HI",
        shogi::PieceType::King => {
            if piece.color == shogi::Color::Black {
                "OU"
            } else {
                "GY"
            }
        }
        shogi::PieceType::ProPawn => "TO",
        shogi::PieceType::ProLance => "NY",
        shogi::PieceType::ProKnight => "NK",
        shogi::PieceType::ProSilver => "NG",
        shogi::PieceType::ProBishop => "UM",
        shogi::PieceType::ProRook => "RY",
    };
    format!("pieces/standard/{}/{prefix}{code}.svg", wallpaper.folder())
}
