use std::sync::{Arc, mpsc, Mutex};

use druid::widget::prelude::*;
use druid::{Data, Lens};

use crate::synth::{Synth, Oscillator, ADSR, Start};
use super::layout::{slider_log};
use super::constants::{WAVEFORMS, DefaultParameter};


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
    pub(super) wave_idx: f64,
    pub(super) volume: f64,
    pub(super) transpose: f64,
    pub(super) tune: f64,
    pub(super) unisons: f64,
    pub(super) env_idx: f64,
}

#[derive(Clone, Data, Lens)]
pub struct EnvSettings {
    pub(super) id: usize,
    pub(super) attack: f64,
    pub(super) decay: f64,
    pub(super) sustain: f64,
    pub(super) release: f64,
}

#[derive(Clone, Data, Lens)]
pub struct SynthUIData {
    #[data(ignore)]
    pub(super) synth: Arc<Mutex<Synth<i16>>>,
    #[data(ignore)]
    pub(super) event_sender: mpsc::Sender<SynthUIEvent>,
    pub(super) octave_modifier: f32,
    pub(super) volume_db: f64,
    pub(super) osc1: OscSettings,
    pub(super) osc2: OscSettings,
    pub(super) env1: EnvSettings,
    pub(super) env2: EnvSettings,
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