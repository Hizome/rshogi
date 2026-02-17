use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::core::game::SoundCue;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};

pub(crate) struct SoundPlayer {
    stream: Option<OutputStream>,
}

impl SoundPlayer {
    pub(crate) fn new() -> Self {
        let stream = OutputStreamBuilder::open_default_stream().ok();
        Self { stream }
    }

    pub(crate) fn play(&self, cue: SoundCue) {
        let Some(stream) = &self.stream else {
            return;
        };
        let path = sound_path(cue);
        let Ok(file) = File::open(path) else {
            return;
        };
        let Ok(source) = Decoder::try_from(BufReader::new(file)) else {
            return;
        };
        let sink = Sink::connect_new(stream.mixer());
        sink.append(source);
        sink.detach();
    }
}

fn sound_path(cue: SoundCue) -> PathBuf {
    let name = match cue {
        SoundCue::Move => "move.ogg",
        SoundCue::Capture => "capture.ogg",
        SoundCue::Error => "error.ogg",
    };
    PathBuf::from("assets")
        .join("sounds")
        .join("lishogi")
        .join("shogi")
        .join(name)
}
