use anyhow::Result;

pub enum Sound {
    Click,
    Star,
}

pub struct SoundManager {}

impl SoundManager {
    pub fn new() -> Result<SoundManager> {
        Ok(SoundManager {})
    }

    pub fn play(&self, sound: Sound) {}
}
