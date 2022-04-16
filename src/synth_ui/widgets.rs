use std::sync::MutexGuard;
use druid::widget::prelude::*;
use druid::widget::{Flex, Slider, CrossAxisAlignment};
use druid::Code as KeyCode;
use druid::KeyEvent;
use super::{
    model::{DefaultParameter, SynthUIData, SynthUIEvent, OscSettings, EnvSettings},
    layout::{slider_log, LOG_SCALE_BASE},
};
use crate::synth::{Synth, WaveForm, ADSRParam};


pub const WAVEFORMS: [WaveFormUI; 5] = [
    WaveFormUI {
        name: "Saw",
        waveform: WaveForm::Saw,
    },
    WaveFormUI {
        name: "Sine",
        waveform: WaveForm::Sine,
    },
    WaveFormUI {
        name: "Square",
        waveform: WaveForm::Square,
    },
    WaveFormUI {
        name: "Pulse25%",
        waveform: WaveForm::Pulse25,
    },
    WaveFormUI {
        name: "Triangle",
        waveform: WaveForm::Triangle,
    },
];

fn round_float(f: f32, accuracy: i32) -> f32 {
    let base = 10f32.powi(accuracy);
    (f * base).round() / base
}

fn get_note(key: &KeyCode) -> Option<f32> {
    let freq = match key {
        KeyCode::KeyZ => 130.81, // C
        KeyCode::KeyS => 138.59, // C#
        KeyCode::KeyX => 146.83, // D
        KeyCode::KeyD => 155.56, // D#
        KeyCode::KeyC => 164.81, // E
        KeyCode::KeyV => 174.61, // F
        KeyCode::KeyG => 185.00, // F#
        KeyCode::KeyB => 196.00, // G
        KeyCode::KeyH => 207.65, // G#
        KeyCode::KeyN => 220.00, // AeE
        KeyCode::KeyJ => 233.08, // A#
        KeyCode::KeyM => 246.94, // B
        _ => return None,
    };
    Some(freq)
}

#[derive(Clone)]
pub struct WaveFormUI {
    pub name: &'static str,
    pub waveform: WaveForm,
}

pub struct SynthUI {
    pub root: Flex<SynthUIData>,
}

impl SynthUI {
    pub fn new() -> Self {
        Self { root: Flex::row().cross_axis_alignment(CrossAxisAlignment::Start) }
    }

    fn handle_key_press(&self, key: &KeyCode, data: &mut SynthUIData) {
        match key {
            KeyCode::ArrowLeft => {
                let modified = round_float(data.octave_modifier / 2.0, 3);
                if modified <= 1.0 / 4.0 {
                    // println!("Lowest octave is active")
                } else {
                    data.octave_modifier = modified
                }
            }
            KeyCode::ArrowRight => {
                let modified = round_float(data.octave_modifier * 2.0, 3);
                if modified >= 16.0 {
                    // println!("Highest octave is active")
                } else {
                    data.octave_modifier = modified
                }
            }
            KeyCode::ArrowUp => {}
            KeyCode::ArrowDown => {}
            KeyCode::KeyU => {}
            _ => match get_note(key) {
                None => {} // println!("Key {:?} is not supported", key),
                Some(freq) => {
                    let mut synth = data.synth.lock().unwrap();
                    if !synth.playing() {
                        data.event_sender.send(SynthUIEvent::NewNotes).unwrap();
                    }
                    synth.note_on(freq * data.octave_modifier, *key)
                }
            }
        }
    }

    fn handle_key_release(&self, key: &KeyCode, data: &mut SynthUIData) {
        if let Some(_) = get_note(key) {
            data.synth.lock().unwrap().note_off(*key);
        }
    }

    fn update_osc(&self, synth: &mut MutexGuard<Synth<i16>>, new: &OscSettings, old: &OscSettings) {
        if new.volume != old.volume {
            synth.set_osc_volume(new.id, new.volume as f32);
        }
        if new.wave_idx != old.wave_idx {
            synth.set_waveform(new.id, &WAVEFORMS[new.wave_idx as usize].waveform);
        }
        if new.transpose != old.transpose {
            synth.set_transpose(new.id, new.transpose as i8);
        }
        if new.tune != old.tune {
            synth.set_tune(new.id, new.tune as i8);
        }
        if new.unisons != old.unisons {
            synth.set_unisons(new.id, new.unisons.round() as usize);
        }
        if new.env_idx != old.env_idx {
            synth.set_env(new.id, new.env_idx.round() as usize);
        }
    }

    fn update_env(&self, synth: &mut MutexGuard<Synth<i16>>, new: &EnvSettings, old: &EnvSettings) {
        if new.attack != old.attack {
            synth.set_env_parameter(new.id, ADSRParam::Attack(LOG_SCALE_BASE.powf(new.attack).round() as f32))
        }
        if new.decay != old.decay {
            synth.set_env_parameter(new.id, ADSRParam::Decay(LOG_SCALE_BASE.powf(new.decay).round() as f32))
        }
        if new.sustain != old.sustain {
            synth.set_env_parameter(new.id, ADSRParam::Sustain(new.sustain as f32))
        }
        if new.release != old.release {
            synth.set_env_parameter(new.id, ADSRParam::Release(LOG_SCALE_BASE.powf(new.release).round() as f32))
        }
    }
}

impl Widget<SynthUIData> for SynthUI {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SynthUIData, env: &Env) {
        match event {
            Event::WindowConnected => {
                if !ctx.is_focused() {
                    ctx.request_focus()
                }
            }
            Event::KeyDown(KeyEvent {
                code,
                repeat,
                ..
            }) => {
                if *code == KeyCode::Escape {
                    ctx.window().close()
                } else if !repeat {
                    self.handle_key_press(code, data)
                }
            }
            Event::KeyUp(KeyEvent {
                code,
                repeat,
                ..
            }) => {
                if !repeat {
                    self.handle_key_release(code, data)
                }
            }
            event => self.root.event(ctx, event, data, env),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &SynthUIData,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => ctx.register_for_focus(),
            _ => {}
        }
        self.root.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old: &SynthUIData,
        new: &SynthUIData,
        env: &Env,
    ) {
        if !new.same(old) {
            if !new.osc1.same(&old.osc1) {
                let mut synth = new.synth.lock().unwrap();
                self.update_osc(&mut synth, &new.osc1, &old.osc1);
            }
            if !new.osc2.same(&old.osc2) {
                let mut synth = new.synth.lock().unwrap();
                self.update_osc(&mut synth, &new.osc2, &old.osc2);
            }
            if new.volume_db != old.volume_db {
                // Slider value is in allowed range
                new.synth.lock().unwrap().set_volume(new.volume_db as i32).unwrap();
            }
            if !new.env1.same(&old.env1) {
                let mut synth = new.synth.lock().unwrap();
                self.update_env(&mut synth, &new.env1, &old.env1);
            }
            if !new.env2.same(&old.env2) {
                let mut synth = new.synth.lock().unwrap();
                self.update_env(&mut synth, &new.env2, &old.env2);
            }
        }
        self.root.update(ctx, old, new, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &SynthUIData,
        env: &Env,
    ) -> Size {
        self.root.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &SynthUIData, env: &Env) {
        self.root.paint(ctx, data, env);
    }
}

pub struct DefaultSlider {
    slider: Slider,
    parameter: DefaultParameter,
}

impl DefaultSlider {
    pub fn new(slider: Slider, parameter: DefaultParameter) -> Self {
        Self {
            slider,
            parameter,
        }
    }
}

impl Widget<f64> for DefaultSlider {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f64, env: &Env) {
        match event {
            Event::MouseDown(e) => {
                if e.button.is_left() && e.mods.ctrl() {
                    match self.parameter {
                        // Log scale parameters
                        DefaultParameter::EnvAttack | DefaultParameter::EnvDecay | DefaultParameter::EnvRelease => {
                            *data = slider_log(self.parameter.default_val() as f32);
                        },
                        _ => *data = self.parameter.default_val(),
                    }
                    return
                }
            },
            _ => {},
        }
        self.slider.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &f64,
        env: &Env,
    ) {
        self.slider.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old: &f64,
        new: &f64,
        env: &Env,
    ) {
        self.slider.update(ctx, old, new, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &f64,
        env: &Env,
    ) -> Size {
        self.slider.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        self.slider.paint(ctx, data, env)
    }
}