use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired, AudioSpecWAV};
use sdl2::AudioSubsystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sound {
    Click,
    Star,
}

const MAX_SOUNDS: usize = 4;

struct SoundCallback {
    playing: Vec<(Vec<u16>, usize)>,
}

impl AudioCallback for SoundCallback {
    type Channel = u16;

    fn callback(&mut self, buffer: &mut [Self::Channel]) {
        for sample in buffer.iter_mut() {
            *sample = 0;
        }

        let playing = std::mem::replace(&mut self.playing, Vec::new());
        for (clip, offset) in playing.into_iter() {
            for (i, sample) in buffer.iter_mut().enumerate() {
                if offset + i >= clip.len() {
                    break;
                }
                *sample += clip[i + offset] / (MAX_SOUNDS as u16);
            }

            let next_offset = offset + buffer.len();
            if next_offset < clip.len() {
                self.playing.push((clip, next_offset));
            }
        }
    }
}

fn load_wav(path: &Path, _spec: &AudioSpec) -> Result<AudioSpecWAV> {
    let wav = AudioSpecWAV::load_wav(path)
        .map_err(|s| anyhow!("unable to load wav {:?}: {}", path, s))?;

    // TODO: Use an audio converter to convert the data.

    /*
    if wav.freq != spec.freq {
        bail!("incorrect frequency: {} vs {}", wav.freq, spec.freq);
    }
    if wav.format != spec.format {
        bail!("incorrect format: {:?} vs {:?}", wav.format, spec.format);
    }
    if wav.channels != spec.channels {
        bail!("incorrect channels: {} vs {}", wav.channels, spec.channels);
    }
    if wav.format != AudioFormat::S16LSB {
        bail!(
            "incorrect format: {:?} vs {:?}",
            wav.format,
            AudioFormat::S16LSB
        );
    }
    */

    if wav.buffer().len() % 2 != 0 {
        bail!("wav parity error");
    }

    Ok(wav)
}

pub struct SoundManager {
    device: AudioDevice<SoundCallback>,
    sounds: HashMap<Sound, AudioSpecWAV>,
}

impl SoundManager {
    pub fn new(audio: &AudioSubsystem) -> Result<SoundManager> {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(512),
        };

        let device = audio
            .open_playback(None, &desired_spec, |_spec| SoundCallback {
                playing: Vec::new(),
            })
            .map_err(|s| anyhow!("error initializing audio device: {}", s))?;

        let sounds = HashMap::new();
        let mut manager = SoundManager { device, sounds };

        manager.load_wav(Sound::Click, "click")?;
        manager.load_wav(Sound::Star, "star")?;

        manager.device.resume();
        Ok(manager)
    }

    fn load_wav(&mut self, sound: Sound, name: &str) -> Result<()> {
        let path_str = format!("./assets/sounds/{}.wav", name);
        let path = Path::new(&path_str);
        let wav = load_wav(path, self.device.spec())?;
        self.sounds.insert(sound, wav);
        Ok(())
    }

    pub fn play(&mut self, sound: Sound) {
        println!("playing sound {:?}", sound);
        let wav = self.sounds.get(&sound).expect("all sounds are in map");
        // TODO: Investigate how difficult it would be to remove this copy.
        let buffer = wav.buffer();
        let mut data = Vec::new();
        for i in 0..buffer.len() {
            if i % 2 == 0 {
                let bytes = [buffer[i], buffer[i + 1]];
                let word = u16::from_le_bytes(bytes);
                data.push(word);
            }
        }

        let mut lock = self.device.lock();
        let callback = lock.deref_mut();
        if callback.playing.len() < MAX_SOUNDS {
            callback.playing.push((data, 0))
        }
    }
}
