use std::error::Error;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::time::Duration;
use rand::prelude::IndexedRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub const UI_SELECT_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/ui_select.ogg"),
]);
pub const UI_ERROR_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/ui_error.ogg"),
]);
pub const UI_DIALOG_OPEN_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/ui_dialog_open.ogg"),
]);

pub const BOOK_OPEN_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/book_open.ogg"),
]);
pub const BOOK_FLIP_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/book_flip.ogg"),
]);

pub const UNDO_REDO_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/undo_redo.ogg"),
]);

pub const NO_PATH_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/no_path.ogg"),
]);
pub const LEVEL_COMPLETE_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/level_complete.ogg"),
]);
pub const LEVEL_PACK_COMPLETE_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/level_pack_complete.ogg"),
]);
pub const LEVEL_RESET: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/level_reset.ogg"),
]);
pub const STEP_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/step_1.ogg"),
    include_bytes!("../../assets/audio/step_2.ogg"),
    include_bytes!("../../assets/audio/step_3.ogg"),
]);

pub const BOX_FALL_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/box_fall.ogg"),
]);
pub const DOOR_OPEN_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/door_open.ogg"),
]);
pub const FLOOR_BROKEN_EFFECT: &SoundEffect = &SoundEffect::new(&[
    include_bytes!("../../assets/audio/floor_broken.ogg"),
]);

pub const BACKGROUND_MUSIC_TRACKS: BackgroundMusicTracks<1> = BackgroundMusicTracks::new([
    &BACKGROUND_MUSIC_FIELDS_OF_ICE,
]);

pub const BACKGROUND_MUSIC_FIELDS_OF_ICE: BackgroundMusic = BackgroundMusic {
    id: BackgroundMusicId::new(1),
    display_name: "Fields of Ice",
    creator: "Jonathan So",
    intro_audio_data: Some(include_bytes!("../../assets/audio/background_music_fields_of_ice_intro.ogg")),
    main_loop_audio_data: include_bytes!("../../assets/audio/background_music_fields_of_ice.ogg"),
};

#[derive(Debug)]
pub struct SoundEffect {
    sound_effects: &'static [&'static [u8]],
}

impl SoundEffect {
    const fn new(sound_effects: &'static [&'static [u8]]) -> Self {
        if sound_effects.is_empty() {
            panic!("At least one sound effect must be present!");
        }

        Self { sound_effects }
    }

    pub fn sound_effects(&self) -> &'static [&'static [u8]] {
        self.sound_effects
    }
}

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

    rand: ChaCha8Rng,
}

impl AudioHandler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let output_stream = OutputStream::try_default();
        let (_stream, stream_handle) = output_stream?;

        let background_music_sink = Sink::try_new(&stream_handle)?;
        let rand = ChaCha8Rng::from_os_rng();

        Ok(Self {
            _stream,

            stream_handle,

            background_music_sink,

            rand,
        })
    }

    pub fn play_sound_effect(&mut self, sound_effect: &'static SoundEffect) -> Result<(), Box<dyn Error>> {
        let sound_effect = *sound_effect.sound_effects.choose(&mut self.rand).unwrap();

        let cursor = Cursor::new(sound_effect);
        let source = Decoder::new(cursor)?.speed(self.rand.random_range(0.99..1.01));

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
