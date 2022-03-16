use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use druid::widget::prelude::*;
use druid::widget::{Flex, Label, Slider, Stepper, CrossAxisAlignment};
// pub use druid::KeyCode;
pub use druid::Code as KeyCode;
use druid::{Data, KeyEvent, Lens, LensExt, WidgetExt};

use crate::synth::waves::WaveForm;
use crate::synth::ADSRParam;
use crate::synth::{Synth, Oscillator, ADSR, Start, adsr_constraints};

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

fn round_float(f: f32, accuracy: i32) -> f32 {
    let base = 10f32.powi(accuracy);
    (f * base).round() / base
}

const LOG_SCALE_BASE: f64 = 2.;

fn slider_log(x: f32) -> f64 {
    f64::log2(x as f64)
}

const DEFAULT_ATTACK: f64 = 300.;
const DEFAULT_DECAY: f64 = 300.;
const DEFAULT_SUSTAIN: f64 = 0.7;
const DEFAULT_RELEASE: f64 = 300.;

#[derive(Clone)]
struct WaveFormUI {
    name: &'static str,
    waveform: WaveForm,
}

const WAVEFORMS: [WaveFormUI; 5] = [
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

#[derive(Clone, Data, Lens)]
struct OscSettings {
    id: usize,
    // title: String,
    wave_idx: f64,
    volume: f64,
    transpose: f64,
    tune: f64,
    unisons: f64,
    env_idx: f64,
}

#[derive(Clone, Data, Lens)]
struct EnvSettings {
    id: usize,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
}

#[derive(Clone, Data, Lens)]
pub struct SynthUIData {
    #[data(ignore)]
    synth: Arc<Mutex<Synth<i16>>>,
    #[data(ignore)]
    event_sender: mpsc::Sender<SynthUIEvent>,
    octave_modifier: f32,
    volume_db: f64,
    osc1: OscSettings,
    osc2: OscSettings,
    env1: EnvSettings,
    env2: EnvSettings,
}

impl SynthUIData {
    pub fn new(synth: Arc<Mutex<Synth<i16>>>, event_sender: mpsc::Sender<SynthUIEvent>, sample_rate: f32) -> Self {
        let mut synth_lock = synth.lock().unwrap();

        // attack, decay and release are log scaler representation now
        let default_attack_log = slider_log(DEFAULT_ATTACK as f32);
        let default_decay_log = slider_log(DEFAULT_DECAY as f32);
        let default_release_log = slider_log(DEFAULT_RELEASE as f32);
        let env1 = EnvSettings {
            id: 0,
            attack: default_attack_log,
            decay: default_decay_log,
            sustain: DEFAULT_SUSTAIN,
            release: default_release_log,
        };
        let envelope1 = ADSR::new(
            sample_rate,
            DEFAULT_ATTACK as u32,
            DEFAULT_DECAY as u32,
            DEFAULT_SUSTAIN as f32,
            DEFAULT_RELEASE as u32);
        synth_lock.add_env(envelope1);

        let env2 = EnvSettings {
            id: 1,
            attack: default_attack_log,
            decay: default_decay_log,
            sustain: DEFAULT_SUSTAIN,
            release: default_release_log,
        };
        let envelope2 = ADSR::new(
            sample_rate,
            DEFAULT_ATTACK as u32,
            DEFAULT_DECAY as u32,
            DEFAULT_SUSTAIN as f32,
            DEFAULT_RELEASE as u32);
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

        drop(synth_lock);
        Self {
            synth,
            event_sender,
            octave_modifier: 2.0,
            volume_db: -36.0,
            osc1,
            osc2,
            env1,
            env2,
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

pub struct SynthUI {
    root: Flex<SynthUIData>,
}

impl SynthUI {
    pub fn new() -> Self {
        Self { root: Flex::row() }
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

const BASIC_LABEL_WITDH: f64 = 80.0;
const LABEL_COLOR: druid::Color = druid::Color::rgba8(0xe9, 0x1e, 0x63, 0xff);
const TEXT_LARGE: f64 = 22.0;
const TEXT_MEDIUM: f64 = 18.0;
const TEXT_SMALL: f64 = 14.0;
const MAX_UNISONS: f64 = 7.0;
const ENV_NUM: f64 = 2.0;
const SLIDER_WIDTH_SMALL: f64 = 110.0;
const SLIDER_WIDTH_MEDIUM: f64 = 170.0;

// unison(label + label + stepper);
fn oscillator_layout<L>(title: &str, osc_lens: L) -> impl Widget<SynthUIData>
where
    L: Lens<SynthUIData, OscSettings>
    + Clone
    + 'static
{
    let left_padding = (10.0, 0.0, 0.0, 0.0);
    let row_padding = (10.0, 0.0, 0.0, 10.0);
    let mut osc_flex = Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(
        Label::new(title).with_text_size(TEXT_MEDIUM).padding(10.0)
    );
    // Volume and envelope
    osc_flex.add_child(Label::new("Volume").with_text_size(TEXT_SMALL).padding(left_padding));
    // Volume slider
    let volume_slider = Slider::new()
                    .with_range(0.0, 1.0)
                    .lens(osc_lens.clone().then(OscSettings::volume)).fix_width(SLIDER_WIDTH_SMALL);
    // Envelope
    let lens_clone = osc_lens.clone();
    let env_idx = Label::dynamic(
        move |data: &SynthUIData, _| {
            ((lens_clone.with(data, |osc| osc.env_idx) + 1.0).round()).to_string()
        }
    ).with_text_size(TEXT_SMALL);
    let env_stepper = Stepper::new()
                    .with_range(0.0, ENV_NUM - 1.0)
                    .with_wraparound(true)
                    .lens(osc_lens.clone().then(OscSettings::env_idx));
    let volume_env_flex = Flex::row()
                    .with_child(volume_slider)
                    .with_child(Label::new("Envelope").with_text_size(TEXT_SMALL))
                    .with_child(env_idx)
                    .with_child(env_stepper);
    osc_flex.add_child(volume_env_flex.padding((0.0, 0.0, 0.0, 10.0)));

    // Waveform
    let lens_clone = osc_lens.clone();
    let wave_label = Label::dynamic(
        move |data: &SynthUIData, _| {
            let idx = lens_clone.with(data, |osc: &OscSettings| { osc.wave_idx });
            WAVEFORMS[idx.round() as usize].name.into()
        }
    );
    let wave_step = Stepper::new()
        .with_range(0.0, (WAVEFORMS.len() - 1) as f64)
        .with_wraparound(true)
        .lens(osc_lens.clone().then(OscSettings::wave_idx));
    let wave_flex = Flex::row().with_child(wave_label.fix_width(100.0)).with_child(wave_step);
    osc_flex.add_child(wave_flex.padding(row_padding));

    // Transpose
    let lens_clone = osc_lens.clone();
    let transpose_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            lens_clone.with(data, |osc: &OscSettings| {
                format!("{} semitones",(osc.transpose as i8))
            })
        }
    ).with_text_size(TEXT_SMALL);
    let transpose_slider = Slider::new()
                        .with_range(-24.0, 24.0)
                        .lens(osc_lens.clone().then(OscSettings::transpose));
    let transpose_flex = Flex::row()
                    .with_child(Label::new("Transpose").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
                    .with_child(transpose_slider.fix_width(SLIDER_WIDTH_MEDIUM))
                    .with_child(transpose_value.fix_width(25.0));
    osc_flex.add_child(transpose_flex.padding(row_padding));

    // Tune
    let lens_clone = osc_lens.clone();
    let tune_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            lens_clone.with(data, |osc: &OscSettings| { 
                format!("{} cents",(osc.tune as i8))
            })
        }
    ).with_text_size(TEXT_SMALL);
    let tune_slider = Slider::new()
                        .with_range(-100.0, 100.0)
                        .lens(osc_lens.clone().then(OscSettings::tune));
    let tune_flex = Flex::row()
                    .with_child(Label::new("Tune").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
                    .with_child(tune_slider.fix_width(SLIDER_WIDTH_MEDIUM))
                    .with_child(tune_value.fix_width(25.0));
    osc_flex.add_child(tune_flex.padding(row_padding));

    // Unisons
    let uni_stepper = Stepper::new()
                    .with_range(1.0, MAX_UNISONS)
                    .with_wraparound(false)
                    .with_step(1.0)
                    .lens(osc_lens.clone().then(OscSettings::unisons));
    let lens_clone = osc_lens.clone();
    let uni_label = Label::dynamic(
        move |data: &SynthUIData, _| {
            lens_clone.with(data, |osc| {
                osc.unisons.round().to_string()
            })
        }
    );
    let uni_flex = Flex::row()
                    .with_child(Label::new("Unisons").with_text_size(TEXT_SMALL))
                    .with_child(uni_label)
                    .with_child(uni_stepper);
    osc_flex.add_child(uni_flex.padding(row_padding));

    osc_flex.padding(5.0).border(druid::Color::BLACK, 1.0).fix_width(370.0)
}

fn synth_volume_layout() -> impl Widget<SynthUIData> {
    let mut volume_flex = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(Label::new("BEEP-BOOP")
                            .with_text_color(LABEL_COLOR)
                            .with_text_size(TEXT_LARGE))
                    .with_spacer(10.0);
    let volume_control = Flex::row()
                .with_child(Label::new("Volume").with_text_size(TEXT_MEDIUM).fix_width(BASIC_LABEL_WITDH))
                .with_child(
                    Slider::new()
                    .with_range(-96.0, -10.0)
                    .lens(SynthUIData::volume_db)
                    .padding(5.0))
                .with_child(
                    Label::dynamic(
                        |data: &SynthUIData, _| {
                            format!("{} dB", data.volume_db.round())
                        }
                    ).fix_width(25.0)
                );

    volume_flex.add_child(volume_control);

    volume_flex.padding(10.0)
}

fn env_layout<L>(title: &str, env_lens: L) -> impl Widget<SynthUIData>
where
    L: Lens<SynthUIData, EnvSettings>
    + Clone
    + 'static
{
    let mut env_flex = Flex::column()
                    .with_child(Label::new(title).with_text_size(TEXT_MEDIUM));

    // Attack
    let lens_clone = env_lens.clone();
    let attack_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            format!("{} ms", lens_clone.with(data, |env| { LOG_SCALE_BASE.powf(env.attack).round() }))
        }
    ).with_text_size(TEXT_SMALL);
    // Log scale slider
    let attack_min = slider_log(adsr_constraints::MIN_ATTACK);
    let attack_max = slider_log(adsr_constraints::MAX_ATTACK);
    let attack_slider = Slider::new()
                    .with_range(attack_min, attack_max)
                    .lens(env_lens.clone().then(EnvSettings::attack));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Attack").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(attack_slider.padding(2.0))
        .with_child(attack_value.fix_width(45.0)).padding(5.0)
    );

    // Decay
    let lens_clone = env_lens.clone();
    let decay_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            format!("{} ms", lens_clone.with(data, |env| { LOG_SCALE_BASE.powf(env.decay).round() }))
        }
    ).with_text_size(TEXT_SMALL);
    // Log scale slider
    let decay_min = slider_log(adsr_constraints::MIN_DECAY);
    let decay_max = slider_log(adsr_constraints::MAX_DECAY);
    let decay_slider = Slider::new()
                    .with_range(decay_min, decay_max)
                    .lens(env_lens.clone().then(EnvSettings::decay));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Decay").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(decay_slider.padding(2.0))
        .with_child(decay_value.fix_width(45.0)).padding(5.0)
    );

    // Sustain
    let lens_clone = env_lens.clone();
    let sustain_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            format!("{:.2}", lens_clone.with(data, |env| { env.sustain }))
        }
    ).with_text_size(TEXT_SMALL);
    let sustain_slider = Slider::new()
                    .with_range(0.0, 1.0)
                    .lens(env_lens.clone().then(EnvSettings::sustain));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Sustain").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(sustain_slider.padding(2.0))
        .with_child(sustain_value.fix_width(45.0)).padding(5.0)
    );

    // Release
    let lens_clone = env_lens.clone();
    let release_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            format!("{} ms", lens_clone.with(data, |env| { LOG_SCALE_BASE.powf(env.release).round() }))
        }
    ).with_text_size(TEXT_SMALL);
    // Log scale slider
    let release_min = slider_log(adsr_constraints::MIN_RELEASE);
    let release_max = slider_log(adsr_constraints::MAX_RELEASE);
    let release_slider = Slider::new()
                    .with_range(release_min, release_max)
                    .lens(env_lens.clone().then(EnvSettings::release));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Release").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(release_slider.padding(2.0))
        .with_child(release_value.fix_width(45.0)).padding(5.0)
    );

    env_flex.padding(15.0).border(druid::Color::BLACK, 1.0).fix_width(330.0)
}

pub fn build_ui() -> impl Widget<SynthUIData> {
    let mut synth_ui = SynthUI::new();

    synth_ui.root.add_child(Flex::column()
                        .with_child(oscillator_layout("Osc1", SynthUIData::osc1))
                        .with_child(oscillator_layout("Osc2", SynthUIData::osc2)));

    let control_layout = Flex::<SynthUIData>::column()
                    .with_child(synth_volume_layout())
                    // .with_spacer(10.0)
                    .with_child(env_layout("Env1", SynthUIData::env1))
                    // .with_spacer(45.0)
                    .with_child(env_layout("Env2", SynthUIData::env2));
    synth_ui.root.add_child(control_layout.expand_height().padding(20.0));

    synth_ui.center()
}
