use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use sdl2::audio::{
    AudioCVT, AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired, AudioSpecWAV,
};
use sdl2::AudioSubsystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sound {
    Click,
    Star,
}

const MAX_SOUNDS: usize = 4;

struct SoundCallback {
    clips: HashMap<Sound, Vec<u8>>,
    playing: Vec<(Sound, usize)>,
}

impl SoundCallback {
    fn load_wav(&mut self, sound: Sound, name: &str, spec: &AudioSpec) -> Result<()> {
        let path_str = format!("./assets/sounds/{}.wav", name);
        let path = Path::new(&path_str);
        let wav = load_wav(path, spec)?;
        self.clips.insert(sound, wav);
        Ok(())
    }
}

impl AudioCallback for SoundCallback {
    type Channel = u8;

    fn callback(&mut self, buffer: &mut [Self::Channel]) {
        for sample in buffer.iter_mut() {
            *sample = 0;
        }

        let playing = std::mem::replace(&mut self.playing, Vec::new());
        for (sound, offset) in playing.into_iter() {
            let clip = self.clips.get(&sound).expect("all sounds should be loaded");

            for (i, sample) in buffer.iter_mut().enumerate() {
                if offset + i >= clip.len() {
                    break;
                }
                *sample += clip[i + offset] / (MAX_SOUNDS as u8);
            }

            let next_offset = offset + buffer.len();
            if next_offset < clip.len() {
                self.playing.push((sound, next_offset));
            }
        }
    }
}

fn load_wav(path: &Path, spec: &AudioSpec) -> Result<Vec<u8>> {
    let wav = AudioSpecWAV::load_wav(path)
        .map_err(|s| anyhow!("unable to load wav {:?}: {}", path, s))?;

    let cvt = AudioCVT::new(
        wav.format,
        wav.channels,
        wav.freq,
        spec.format,
        spec.channels,
        spec.freq,
    )
    .map_err(|s| anyhow!("unable to create audio converter: {}", s))?;

    let buffer = cvt.convert(wav.buffer().into());

    if wav.buffer().len() % 2 != 0 {
        bail!("wav parity error");
    }

    Ok(buffer)
}

pub struct SoundManager {
    device: AudioDevice<SoundCallback>,
}

impl SoundManager {
    pub fn new(audio: &AudioSubsystem) -> Result<SoundManager> {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(512),
        };

        let mut device = audio
            .open_playback(None, &desired_spec, |_spec| SoundCallback {
                clips: HashMap::new(),
                playing: Vec::new(),
            })
            .map_err(|s| anyhow!("error initializing audio device: {}", s))?;

        SoundManager::load_sounds(&mut device)?;

        device.resume();
        Ok(SoundManager { device })
    }

    fn load_sounds(device: &mut AudioDevice<SoundCallback>) -> Result<()> {
        let spec = device.spec().clone();
        let mut lock = device.lock();
        let callback = lock.deref_mut();
        callback.load_wav(Sound::Click, "click", &spec)?;
        callback.load_wav(Sound::Star, "star", &spec)?;
        Ok(())
    }

    pub fn play(&mut self, sound: Sound) {
        println!("playing sound {:?}", sound);
        let mut lock = self.device.lock();
        let callback = lock.deref_mut();
        if callback.playing.len() < MAX_SOUNDS {
            callback.playing.push((sound, 0));
        }
    }
}
