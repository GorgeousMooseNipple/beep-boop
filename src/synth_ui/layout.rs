use druid::{Lens, LensExt, WidgetExt};
use druid::widget::prelude::*;
use druid::widget::{Flex, Stepper, Slider, Label, CrossAxisAlignment};

use super::model::{SynthUIData, OscSettings, EnvSettings, WAVEFORMS, ClickableSlider, DefaultParameter};
use crate::synth::adsr_constraints;


pub const LOG_SCALE_BASE: f64 = 2.;

const BASIC_LABEL_WITDH: f64 = 80.0;
const LABEL_COLOR_MAIN: druid::Color = druid::Color::rgba8(0xe9, 0x1e, 0x63, 0xff);
const LABEL_COLOR_SECONDARY: druid::Color = druid::Color::rgba8(0x35, 0xaa, 0xee, 0xff);
const BORDER_COLOR: druid::Color = druid::Color::rgba8(0x03, 0x12, 0x14, 0xff);
pub const BACKGROUND_COLOR: druid::Color = druid::Color::rgba8(0x29, 0x29, 0x29, 0xff);
const TEXT_LARGE: f64 = 22.0;
const TEXT_MEDIUM: f64 = 18.0;
const TEXT_SMALL: f64 = 14.0;
const MAX_UNISONS: f64 = 7.0;
const ENV_NUM: f64 = 2.0;
const SLIDER_WIDTH_SMALL: f64 = 110.0;
const SLIDER_WIDTH_MEDIUM: f64 = 170.0;


pub fn slider_log(x: f32) -> f64 {
    f64::log2(x as f64)
}

// unison(label + label + stepper);
pub fn oscillator_layout<L>(title: &str, osc_lens: L) -> impl Widget<SynthUIData>
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
        Label::new(title).with_text_size(TEXT_MEDIUM).with_text_color(LABEL_COLOR_SECONDARY).padding(10.0)
    );
    // Volume and envelope
    osc_flex.add_child(Label::new("Volume").with_text_size(TEXT_SMALL).padding(left_padding));
    // Volume slider
    let volume_slider = ClickableSlider::new(Slider::new()
                    .with_range(0.0, 1.0), DefaultParameter::OscVolume)
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
    let transpose_slider = ClickableSlider::new(Slider::new()
                        .with_range(-24.0, 24.0), DefaultParameter::OscTranspose)
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
    let tune_slider = ClickableSlider::new(Slider::new()
                        .with_range(-100.0, 100.0), DefaultParameter::OscTune)
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

    osc_flex.padding(5.0).border(BORDER_COLOR, 1.0).fix_width(390.0)
}

pub fn synth_volume_layout() -> impl Widget<SynthUIData> {
    let mut volume_flex = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(Label::new("BEEP-BOOP")
                            .with_text_color(LABEL_COLOR_MAIN)
                            .with_text_size(TEXT_LARGE))
                    .with_spacer(10.0);
    let volume_control = Flex::row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(Label::new("Volume").with_text_size(TEXT_MEDIUM).fix_width(BASIC_LABEL_WITDH))
                .with_child(
                    Slider::new()
                    .with_range(-96.0, 0.0)
                    .lens(SynthUIData::volume_db)
                    .padding((5.0, 0.0, 5.0, 0.0))
                    .fix_width(SLIDER_WIDTH_SMALL))
                .with_child(
                    Label::dynamic(
                        |data: &SynthUIData, _| {
                            format!("{} dB", data.volume_db.round())
                        }
                    ).fix_width(25.0)
                );

    volume_flex.add_child(volume_control);

    volume_flex
}

pub fn env_layout<L>(title: &str, env_lens: L) -> impl Widget<SynthUIData>
where
    L: Lens<SynthUIData, EnvSettings>
    + Clone
    + 'static
{
    let mut env_flex = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(Label::new(title).with_text_size(TEXT_MEDIUM).padding(5.0));

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
    let attack_slider = ClickableSlider::new(Slider::new()
                    .with_range(attack_min, attack_max), DefaultParameter::EnvAttack)
                    .lens(env_lens.clone().then(EnvSettings::attack));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Attack").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(attack_slider.padding(2.0).fix_width(SLIDER_WIDTH_MEDIUM))
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
    let decay_slider = ClickableSlider::new(Slider::new()
                    .with_range(decay_min, decay_max), DefaultParameter::EnvDecay)
                    .lens(env_lens.clone().then(EnvSettings::decay));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Decay").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(decay_slider.padding(2.0).fix_width(SLIDER_WIDTH_MEDIUM))
        .with_child(decay_value.fix_width(45.0)).padding(5.0)
    );

    // Sustain
    let lens_clone = env_lens.clone();
    let sustain_value = Label::dynamic(
        move |data: &SynthUIData, _| {
            format!("{:.2}", lens_clone.with(data, |env| { env.sustain }))
        }
    ).with_text_size(TEXT_SMALL);
    let sustain_slider = ClickableSlider::new(Slider::new()
                    .with_range(0.0, 1.0), DefaultParameter::EnvSustain)
                    .lens(env_lens.clone().then(EnvSettings::sustain));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Sustain").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(sustain_slider.padding(2.0).fix_width(SLIDER_WIDTH_MEDIUM))
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
    let release_slider = ClickableSlider::new(Slider::new()
                    .with_range(release_min, release_max), DefaultParameter::EnvRelease)
                    .lens(env_lens.clone().then(EnvSettings::release));
    env_flex.add_child(
        Flex::row()
        .with_child(Label::new("Release").with_text_size(TEXT_SMALL).fix_width(BASIC_LABEL_WITDH))
        .with_child(release_slider.padding(2.0).fix_width(SLIDER_WIDTH_MEDIUM))
        .with_child(release_value.fix_width(45.0)).padding(5.0)
    );

    env_flex.padding(15.0).fix_width(360.0)
}