use std::error::Error;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::time::Duration;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub const UI_SELECT_EFFECT: &[u8] = include_bytes!("../../assets/audio/ui_select.ogg");
pub const UI_ERROR_EFFECT: &[u8] = include_bytes!("../../assets/audio/ui_error.ogg");
pub const UI_DIALOG_OPEN_EFFECT: &[u8] = include_bytes!("../../assets/audio/ui_dialog_open.ogg");

pub const BOOK_OPEN_EFFECT: &[u8] = include_bytes!("../../assets/audio/book_open.ogg");
pub const BOOK_FLIP_EFFECT: &[u8] = include_bytes!("../../assets/audio/book_flip.ogg");

pub const UNDO_REDO_EFFECT: &[u8] = include_bytes!("../../assets/audio/undo_redo.ogg");

pub const SECRET_FOUND_EFFECT: &[u8] = include_bytes!("../../assets/audio/secret_found.ogg");
pub const NO_PATH_EFFECT: &[u8] = include_bytes!("../../assets/audio/no_path.ogg");
pub const LEVEL_COMPLETE_EFFECT: &[u8] = include_bytes!("../../assets/audio/level_complete.ogg");
pub const LEVEL_PACK_COMPLETE_EFFECT: &[u8] = include_bytes!("../../assets/audio/level_pack_complete.ogg");
pub const LEVEL_RESET: &[u8] = include_bytes!("../../assets/audio/level_reset.ogg");
pub const STEP_EFFECT: &[u8] = include_bytes!("../../assets/audio/step.ogg");

pub const BOX_FALL_EFFECT: &[u8] = include_bytes!("../../assets/audio/box_fall.ogg");
pub const DOOR_OPEN_EFFECT: &[u8] = include_bytes!("../../assets/audio/door_open.ogg");
pub const FLOOR_BROKEN_EFFECT: &[u8] = include_bytes!("../../assets/audio/floor_broken.ogg");

pub const BACKGROUND_MUSIC_TRACKS: BackgroundMusicTracks<6> = BackgroundMusicTracks::new([
    &BACKGROUND_MUSIC_FIELDS_OF_ICE,
    &BACKGROUND_MUSIC_LEAP,
    &BACKGROUND_MUSIC_TRIANGULAR,
    &BACKGROUND_MUSIC_LONELY_NIGHT,
    &BACKGROUND_MUSIC_RESOW,
    &BACKGROUND_MUSIC_CATCHY,
]);

pub const BACKGROUND_MUSIC_FIELDS_OF_ICE: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(1),
    display_name: "Fields of Ice",
    creator: "Jonathan So",
    intro_audio_data: Some(include_bytes!("../../assets/audio/background_music_fields_of_ice_intro.ogg")),
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_fields_of_ice.ogg"),
};

pub const BACKGROUND_MUSIC_LEAP: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(2),
    display_name: "Leap [8bit]",
    creator: "nene",
    intro_audio_data: None,
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_leap.ogg"),
};

pub const BACKGROUND_MUSIC_TRIANGULAR: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(3),
    display_name: "Triangular Ideology: The Fan Sequel",
    creator: "Spring Spring",
    intro_audio_data: None,
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_triangular.ogg"),
};

pub const BACKGROUND_MUSIC_LONELY_NIGHT: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(4),
    display_name: "Lonely Night",
    creator: "Centurion_of_war",
    intro_audio_data: None,
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_lonely_night.ogg"),
};

pub const BACKGROUND_MUSIC_RESOW: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(5),
    display_name: "Re-Sow",
    creator: "Chasersgaming",
    intro_audio_data: None,
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_resow.ogg"),
};

pub const BACKGROUND_MUSIC_CATCHY: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(6),
    display_name: "Catchy",
    creator: "Spring Spring",
    intro_audio_data: None,
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_catchy.ogg"),
};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BackgroundMusicId(NonZeroUsize);

impl BackgroundMusicId {
    const fn new(id: usize) -> BackgroundMusicId {
        BackgroundMusicId(NonZeroUsize::new(id).unwrap())
    }

    pub fn id(self) -> usize {
        self.0.get()
    }
}

#[derive(Debug)]
pub struct BackgroundMusic {
    id: BackgroundMusicId,
    display_name: &'static str,
    creator: &'static str,
    intro_audio_data: Option<&'static [u8]>,
    main_loop_audio_data: &'static [u8],
}

impl BackgroundMusic {
    pub fn id(&self) -> BackgroundMusicId {
        self.id
    }

    pub fn display_name(&self) -> &'static str {
        self.display_name
    }

    pub fn creator(&self) -> &'static str {
        self.creator
    }

    pub fn intro_audio_data(&self) -> Option<&'static [u8]> {
        self.intro_audio_data
    }

    pub fn main_loop_audio_data(&self) -> &'static [u8] {
        self.main_loop_audio_data
    }
}

#[derive(Debug)]
pub struct BackgroundMusicTracks<const N: usize> {
    tracks: [&'static BackgroundMusic; N],
}

impl<const N: usize> BackgroundMusicTracks<N> {
    const fn new(tracks: [&'static BackgroundMusic; N]) -> BackgroundMusicTracks<N> {
        BackgroundMusicTracks {
            tracks,
        }
    }

    pub fn check_id(&self, id: usize) -> Option<BackgroundMusicId> {
        for track in &self.tracks {
            if track.id.id() == id {
                return Some(track.id);
            }
        }

        None
    }

    pub fn get_track_by_id(&self, id: BackgroundMusicId) -> &BackgroundMusic {
        self.tracks.iter().find(|background_music| background_music.id == id).unwrap()
    }

    pub fn tracks(&self) -> &[&BackgroundMusic] {
        &self.tracks
    }
}

pub struct AudioHandler {
    _stream: OutputStream,

    stream_handle: OutputStreamHandle,

    background_music_sink: Sink,
}

impl AudioHandler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let output_stream = OutputStream::try_default();
        let (_stream, stream_handle) = output_stream?;

        let background_music_sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            _stream,

            stream_handle,

            background_music_sink
        })
    }

    pub fn play_sound_effect(&self, sound_effect: &'static [u8]) -> Result<(), Box<dyn Error>> {
        let cursor = Cursor::new(sound_effect);
        let source = Decoder::new(cursor)?;

        self.stream_handle.play_raw(source.convert_samples())?;

        Ok(())
    }

    pub fn stop_background_music(&self) {
        self.background_music_sink.stop();
    }

    pub fn set_background_music_loop(&self, intro: Option<&'static [u8]>, main_loop: &'static [u8]) -> Result<(), Box<dyn Error>> {
        self.stop_background_music();

        if let Some(intro) = intro {
            let cursor = Cursor::new(intro);
            let decoder = Decoder::new(cursor)?;
            let source = decoder.fade_in(Duration::from_secs(1));
            self.background_music_sink.append(source);
        }

        let cursor = Cursor::new(main_loop);
        let decoder = Decoder::new_looped(cursor)?;
        if intro.is_some() {
            self.background_music_sink.append(decoder);
        }else {
            let source = decoder.fade_in(Duration::from_secs(1));

            self.background_music_sink.append(source);
        }

        Ok(())
    }
}
