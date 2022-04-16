use std::sync::{Arc, mpsc, Mutex};

use druid::widget::prelude::*;
use druid::{Data, Lens};

use crate::synth::{Synth, Oscillator, ADSR, Start};
use super::layout::{slider_log};
use super::widgets::WAVEFORMS;


const DEFAULT_ATTACK: f64 = 300.;
const DEFAULT_DECAY: f64 = 300.;
const DEFAULT_SUSTAIN: f64 = 0.7;
const DEFAULT_RELEASE: f64 = 300.;
const DEFAULT_TRANSPOSE: f64 = 0.0;
const DEFAULT_TUNE: f64 = 0.0;
const DEFAULT_OSC_VOLUME: f64 = 0.5;

pub enum DefaultParameter {
    EnvAttack,
    EnvDecay,
    EnvSustain,
    EnvRelease,
    OscTranspose,
    OscTune,
    OscVolume,
}

impl DefaultParameter {
    pub fn default_val(&self) -> f64 {
        match self {
            DefaultParameter::EnvAttack => DEFAULT_ATTACK,
            DefaultParameter::EnvDecay => DEFAULT_DECAY,
            DefaultParameter::EnvSustain => DEFAULT_SUSTAIN,
            DefaultParameter::EnvRelease => DEFAULT_RELEASE,
            DefaultParameter::OscTranspose => DEFAULT_TRANSPOSE,
            DefaultParameter::OscTune => DEFAULT_TUNE,
            DefaultParameter::OscVolume => DEFAULT_OSC_VOLUME,
        }
    }
}

use druid::{DelegateCtx, WindowId};

pub enum SynthUIEvent {
    NewNotes,
    WindowClosed,
}

pub struct Delegate;

impl druid::AppDelegate<SynthUIData> for Delegate {
    fn window_removed(
        &mut self,
        _id: WindowId,
        data: &mut SynthUIData,
        _env: &Env,
        _ctx: &mut DelegateCtx
    ) {
        data.event_sender.send(SynthUIEvent::WindowClosed).unwrap();
    }
}

#[derive(Clone, Data, Lens)]
pub struct OscSettings {
    pub id: usize,
    // title: String,
    pub wave_idx: f64,
    pub volume: f64,
    pub transpose: f64,
    pub tune: f64,
    pub unisons: f64,
    pub env_idx: f64,
}

#[derive(Clone, Data, Lens)]
pub struct EnvSettings {
    pub id: usize,
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
}

#[derive(Clone, Data, Lens)]
pub struct SynthUIData {
    #[data(ignore)]
    pub synth: Arc<Mutex<Synth<i16>>>,
    #[data(ignore)]
    pub event_sender: mpsc::Sender<SynthUIEvent>,
    pub octave_modifier: f32,
    pub volume_db: f64,
    pub osc1: OscSettings,
    pub osc2: OscSettings,
    pub env1: EnvSettings,
    pub env2: EnvSettings,
}

impl SynthUIData {
    pub fn new(synth: Arc<Mutex<Synth<i16>>>, event_sender: mpsc::Sender<SynthUIEvent>, sample_rate: f32) -> Self {
        let mut synth_lock = synth.lock().unwrap();

        // attack, decay and release are log scaler representation now
        let default_attack_log = slider_log(DefaultParameter::EnvAttack.default_val() as f32);
        let default_decay_log = slider_log(DefaultParameter::EnvDecay.default_val() as f32);
        let default_release_log = slider_log(DefaultParameter::EnvRelease.default_val() as f32);
        let env1 = EnvSettings {
            id: 0,
            attack: default_attack_log,
            decay: default_decay_log,
            sustain: DefaultParameter::EnvSustain.default_val(),
            release: default_release_log,
        };
        let envelope1 = ADSR::new(
            sample_rate,
            DefaultParameter::EnvAttack.default_val() as u32,
            DefaultParameter::EnvDecay.default_val() as u32,
            DefaultParameter::EnvSustain.default_val() as f32,
            DefaultParameter::EnvRelease.default_val() as u32);
        synth_lock.add_env(envelope1);

        let env2 = EnvSettings {
            id: 1,
            attack: default_attack_log,
            decay: default_decay_log,
            sustain: DefaultParameter::EnvSustain.default_val(),
            release: default_release_log,
        };
        let envelope2 = ADSR::new(
            sample_rate,
            DefaultParameter::EnvAttack.default_val() as u32,
            DefaultParameter::EnvDecay.default_val() as u32,
            DefaultParameter::EnvSustain.default_val() as f32,
            DefaultParameter::EnvRelease.default_val() as u32);
        synth_lock.add_env(envelope2);

        let osc1 = OscSettings {
            id: 0,
            wave_idx: 0.0,
            volume: 0.3,
            transpose: 0.0,
            tune: 15.0,
            unisons: 3.0,
            env_idx: 0.0,
        };
        let mut oscillator1 = Oscillator::new(
            sample_rate,
            WAVEFORMS[osc1.wave_idx as usize].waveform.clone(),
            osc1.env_idx as usize,
            osc1.volume as f32);
        oscillator1.set_start(Start::Soft);
        oscillator1.tune(osc1.tune as i8);
        oscillator1.transpose(osc1.transpose as i8);
        oscillator1.set_unison_num(osc1.unisons as usize);
        synth_lock.add_osc(oscillator1);
        let osc2 = OscSettings {
            id: 1,
            wave_idx: 1.0,
            volume: 0.5,
            transpose: -12.0,
            tune: 0.0,
            unisons: 1.0,
            env_idx: 0.0,
        };
        let mut oscillator2 = Oscillator::new(
            sample_rate,
            WAVEFORMS[osc2.wave_idx as usize].waveform.clone(),
            osc2.env_idx as usize,
            osc2.volume as f32);
        oscillator2.set_start(Start::Soft);
        oscillator2.tune(osc2.tune as i8);
        oscillator2.transpose(osc2.transpose as i8);
        oscillator2.set_unison_num(osc2.unisons as usize);
        synth_lock.add_osc(oscillator2);

        let volume_db = -25.0;
        synth_lock.set_volume(volume_db as i32).unwrap();
        drop(synth_lock);
        Self {
            synth,
            event_sender,
            octave_modifier: 2.0,
            volume_db,
            osc1,
            osc2,
            env1,
            env2,
        }
    }
}