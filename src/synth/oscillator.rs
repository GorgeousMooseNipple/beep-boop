use std::time::Instant;

use super::envelope::ADSR;
use super::waves::{Wave, WaveForm};
use super::{KeyCode, Note, Released};

#[derive(Debug)]
struct Unison {
    freq_mod: f32,
    volume: f32,
}

#[derive(Debug)]
struct UnisonVoice {
    phase: f32,
    phase_incr: f32,
    volume: f32,
}

#[derive(Debug)]
pub struct Voice {
    note: Note,
    volume: f32,
    unisons: Vec<UnisonVoice>,
}

#[allow(dead_code)]
pub enum Start {
    Soft,
    Hard,
    Random,
}

enum PhaseStart {
    Soft,
    Hard(f32),
    Random(f32),
}

impl PhaseStart {
    fn value(&self) -> f32 {
        match self {
            PhaseStart::Soft => 0.0,
            PhaseStart::Hard(period) => period / 4.0,
            PhaseStart::Random(period) => period * rand::random::<f32>(),
        }
    }

    fn change_period(&mut self, period: f32) {
        match self {
            PhaseStart::Soft => {}
            PhaseStart::Hard(_) => *self = PhaseStart::Random(period),
            PhaseStart::Random(_) => *self = PhaseStart::Random(period),
        }
    }
}

// Panning TODO:
// pan value == 0.0 - full left; == 1.0 - full right
// left = value * sin((1- pan) * PI / 2)
// right = value * sin(pan * PI / 2)
pub struct Oscillator {
    sample_rate: f32,
    wave: Box<dyn Wave + Send>,
    pub waveform: WaveForm,
    pub env_idx: usize,
    pub volume: f32,
    voices: Vec<Voice>,
    panning: f32,
    pub transpose: f32,
    pub tune: f32,
    unisons: Vec<Unison>,
    phase_start: PhaseStart,
}

impl Oscillator {
    pub fn new(sample_rate: f32, waveform: WaveForm, env_idx: usize, volume: f32) -> Self {
        let volume = volume.min(1.0).max(0.0);
        Self {
            sample_rate: sample_rate,
            wave: waveform.get_wave(),
            waveform: waveform,
            env_idx: env_idx,
            volume: volume,
            voices: Vec::new(),
            panning: 0.0,
            transpose: 1.0,
            tune: 1.0,
            unisons: vec![Unison {
                freq_mod: 1.0,
                volume: 1.0,
            }],
            phase_start: PhaseStart::Soft,
        }
    }

    pub fn create_voice(&mut self, note: &Note) {
        if let None = self
            .voices
            .iter()
            .find(|v| v.note == *note && v.note.released.is_none())
        {
            let phase_incr = note.frequency / self.sample_rate * self.transpose;
            let mut unisons = Vec::<UnisonVoice>::with_capacity(7);
            let period = self.wave.period();
            let mut uni_iter = self.unisons.iter();
            if self.unisons.len() % 2 == 1 {
                // At least one "unison" is always present
                let central_uni = uni_iter.next().unwrap();
                unisons.push(UnisonVoice {
                    phase: self.phase_start.value(),
                    phase_incr: phase_incr * central_uni.freq_mod,
                    volume: central_uni.volume,
                });
            }
            for uni in uni_iter {
                unisons.push(UnisonVoice {
                    phase: period * rand::random::<f32>(),
                    phase_incr: phase_incr * uni.freq_mod,
                    volume: uni.volume,
                })
            }
            self.voices.push(Voice {
                note: note.clone(),
                volume: 0.0,
                unisons: unisons,
            });
        }
    }

    pub fn voice_off(&mut self, key: KeyCode) {
        if let Some(Voice { note, volume, .. }) = self
            .voices
            .iter_mut()
            .find(|v| v.note.triggered_by == key && v.note.released.is_none())
        {
            note.released = Some(Released {
                time: Instant::now(),
                value: *volume,
            })
        }
    }

    pub fn get_sample(&mut self, adsr: &ADSR) -> f32 {
        let mut sample = 0.0;
        let mut muted_voices = false;
        for Voice {
            note,
            volume,
            unisons,
        } in self.voices.iter_mut()
        {
            *volume = adsr.get_volume_incr(volume, &note.triggered_time, &note.released);
            *volume = volume.min(1.0);
            if *volume <= 0.01 {
                muted_voices = true;
                continue;
            }
            let mut voice_sample = 0.0;
            for uni in unisons.iter_mut() {
                voice_sample += self.wave.wave_func(uni.phase) * uni.volume;
                uni.phase = self.wave.next_phase(uni.phase, uni.phase_incr);
            }
            sample += voice_sample * *volume;
        }
        if muted_voices {
            self.voices
                .retain(|v| !(v.note.released.is_some() && v.volume <= 0.01));
        }
        sample * self.volume
    }

    pub fn set_waveform(&mut self, waveform: &WaveForm) {
        self.waveform = waveform.clone();
        self.wave = waveform.get_wave();
        self.phase_start.change_period(self.wave.period());
    }

    pub fn set_start(&mut self, start: Start) {
        match start {
            Start::Soft => self.phase_start = PhaseStart::Soft,
            Start::Hard => self.phase_start = PhaseStart::Hard(self.wave.period()),
            Start::Random => self.phase_start = PhaseStart::Random(self.wave.period()),
        }
    }

    // Semitones
    pub fn transpose(&mut self, semitones: i8) {
        let transpose = 2f32.powf(semitones as f32 / 12.0);
        for Voice { unisons, .. } in self.voices.iter_mut() {
            for UnisonVoice { phase_incr, .. } in unisons.iter_mut() {
                *phase_incr = *phase_incr / self.transpose * transpose;
            }
        }
        self.transpose = transpose;
    }

    // Cents
    pub fn tune(&mut self, cents: i8) {
        self.tune = 2f32.powf(cents as f32 / (12.0 * 100.0));
        self.update_unison();
    }

    fn update_unison(&mut self) {
        self.set_unison_num(self.unisons.len());
    }

    pub fn set_unison_num(&mut self, num: usize) {
        self.unisons.clear();
        if num <= 1 {
            self.unisons.push(Unison {
                freq_mod: self.tune,
                volume: 1.0,
            });
        } else {
            if num % 2 == 1 {
                self.unisons.push(Unison {
                    freq_mod: 1.0,
                    volume: 1.0,
                })
            }
            let pairs_num = (num - num % 2) / 2;
            // Do better
            let max_volume = 0.7;
            let volume_step = max_volume / pairs_num as f32;
            for i in 0..pairs_num {
                let fraction: f32 = (pairs_num - i) as f32 / pairs_num as f32;
                let volume = volume_step * (pairs_num - i) as f32;
                let freq_mod = self.tune.powf(fraction);
                self.unisons.push(Unison {
                    freq_mod: freq_mod,
                    volume: volume,
                });
                // Detune in other direction
                self.unisons.push(Unison {
                    freq_mod: 1.0 / freq_mod,
                    volume: volume,
                });
            }
        }
        // Update for existing voices
        let period = self.wave.period();
        for Voice { note, unisons, .. } in self.voices.iter_mut() {
            let phase_incr = note.frequency * self.transpose / self.sample_rate;
            let phases: Vec<f32> = unisons.iter().map(|u| u.phase).collect();
            unisons.clear();
            for i in 0..self.unisons.len() {
                let phase: f32;
                if i < phases.len() {
                    phase = phases[i];
                } else {
                    phase = period * rand::random::<f32>();
                }
                unisons.push(UnisonVoice {
                    phase: phase,
                    phase_incr: phase_incr * self.unisons[i].freq_mod,
                    volume: self.unisons[i].volume,
                })
            }
        }
        //
    }

    pub fn has_active_voices(&self) -> bool {
        !self.voices.is_empty()
    }
}
