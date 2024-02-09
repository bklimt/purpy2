#[cfg(feature = "sdl2")]
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sound {
    Click = 0,
    Star,
}

pub trait SoundPlayer {
    fn play(&mut self, sound: Sound);
}

pub struct NoopSoundPlayer {}

impl SoundPlayer for NoopSoundPlayer {
    fn play(&mut self, _sound: Sound) {}
}

pub struct SoundManager {
    internal: Box<dyn SoundPlayer>,
}

impl SoundManager {
    fn with_internal(internal: Box<dyn SoundPlayer>) -> SoundManager {
        Self { internal }
    }

    pub fn noop_manager() -> SoundManager {
        Self::with_internal(Box::new(NoopSoundPlayer {}))
    }

    #[cfg(feature = "sdl2")]
    pub fn with_sdl(audio: &sdl2::AudioSubsystem) -> Result<Self> {
        Ok(Self::with_internal(Box::new(
            crate::sdl::sdlsoundmanager::SdlSoundManager::new(audio)?,
        )))
    }

    pub fn play(&mut self, sound: Sound) {
        self.internal.play(sound)
    }
}
