mod envelope;
mod oscillator;
pub mod waves;

pub use self::envelope::{ADSR, ADSRParam, adsr_constraints};
pub use self::oscillator::{Oscillator, Start};
pub use self::waves::WaveForm;
use crate::error::{BaseError, Result};
pub use crate::synth_ui::KeyCode;

use std::time::Instant;

#[allow(non_camel_case_types)]
type dB = i32;

pub trait SampleFormat:
    portaudio_rs::stream::SampleType
    + num_traits::AsPrimitive<f32>
    + num_traits::Bounded
    + num_traits::FromPrimitive
    + std::cmp::PartialOrd
{
}

impl SampleFormat for u8 {}
impl SampleFormat for i8 {}
impl SampleFormat for i16 {}
impl SampleFormat for i32 {}
impl SampleFormat for f32 {}

#[derive(Debug, Clone)]
pub struct Released {
    pub time: Instant,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct Note {
    frequency: f32,
    triggered_by: KeyCode,
    triggered_time: Instant,
    released: Option<Released>,
}

impl Note {
    pub fn new(frequency: f32, key: KeyCode) -> Self {
        Self {
            frequency: frequency,
            triggered_by: key,
            triggered_time: Instant::now(),
            released: None,
        }
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency && self.triggered_by == other.triggered_by
    }
}

pub struct Synth<SampleType: SampleFormat> {
    sample_rate: f32,
    volume: f32,
    pub oscillators: Vec<Oscillator>,
    pub envelopes: Vec<ADSR>,
    _sample_type: std::marker::PhantomData<SampleType>,
}

impl<SampleType: SampleFormat> Synth<SampleType> {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate,
            volume: 1024.0,
            oscillators: Vec::new(),
            envelopes: Vec::new(),
            _sample_type: std::marker::PhantomData,
        }
    }

    pub fn add_osc(&mut self, osc: Oscillator) {
        self.oscillators.push(osc)
    }

    pub fn add_env(&mut self, env: ADSR) {
        self.envelopes.push(env)
    }

    pub fn set_unisons(&mut self, osc_idx: usize, num: usize) {
        self.oscillators[osc_idx].set_unison_num(num);
    }

    pub fn set_transpose(&mut self, osc_idx: usize, semitones: i8) {
        self.oscillators[osc_idx].transpose(semitones);
    }

    pub fn set_tune(&mut self, osc_idx: usize, cents: i8) {
        self.oscillators[osc_idx].tune(cents);
    }

    pub fn set_volume(&mut self, volume: dB) -> Result<()> {
        if volume > 0 || volume < -96 {
            return Err(BaseError::SynthError(
                "[-96, 0] dB is the range for volume".to_owned(),
            ));
        }
        self.volume = SampleType::max_value().as_() * 10f32.powf(volume as f32 / 20.0);
        Ok(())
    }

    pub fn set_osc_volume(&mut self, osc_idx: usize, volume: f32) {
        self.oscillators[osc_idx].volume = volume;
    }

    pub fn note_on(&mut self, freq: f32, key: KeyCode) {
        let note = Note::new(freq, key);
        self.oscillators
            .iter_mut()
            .for_each(|osc| osc.create_voice(&note))
    }

    pub fn note_off(&mut self, key: KeyCode) {
        self.oscillators
            .iter_mut()
            .for_each(|osc| osc.voice_off(key))
    }

    pub fn playing(&self) -> bool {
        self.oscillators.iter().any(|osc| osc.has_active_voices())
    }

    pub fn set_waveform(&mut self, osc_idx: usize, waveform: &WaveForm) {
        self.oscillators[osc_idx].set_waveform(waveform);
    }

    pub fn set_env_parameter(&mut self, env_idx: usize, param: ADSRParam) {
        self.envelopes[env_idx].set_parameter(param);
    }

    pub fn set_env(&mut self, osc_idx: usize, env_idx: usize) {
        self.oscillators[osc_idx].env_idx = env_idx;
    }
}

impl<SampleType: SampleFormat> Iterator for Synth<SampleType> {
    type Item = SampleType;

    fn next(&mut self) -> Option<Self::Item> {
        // let sample = self
        //     .oscillators
        //     .iter_mut()
        //     .map(|osc| osc.get_sample())
        //     .sum::<f32>()
        //     * self.volume;
        let mut sample: f32 = 0.0;
        for osc in self.oscillators.iter_mut() {
            sample += osc.get_sample(&self.envelopes[osc.env_idx]);
        }
        Some(SampleType::from_f32(sample * self.volume).unwrap())
    }
}
