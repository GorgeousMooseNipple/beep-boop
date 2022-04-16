mod model;
mod layout;
mod widgets;

pub use druid::Code as KeyCode;
use druid::widget::prelude::*;
use druid::widget::{Flex,CrossAxisAlignment};
use druid::{WidgetExt};

pub use model::{SynthUIData, SynthUIEvent, Delegate};
use widgets::SynthUI;
use layout::{BACKGROUND_COLOR, oscillator_layout, synth_volume_layout, env_layout};


pub fn build_ui() -> impl Widget<SynthUIData> {
    let mut synth_ui = SynthUI::new();

    synth_ui.root.add_child(Flex::column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(oscillator_layout("Osc1", SynthUIData::osc1))
                        .with_spacer(10.0)
                        .with_child(oscillator_layout("Osc2", SynthUIData::osc2)));

    let control_layout = Flex::<SynthUIData>::column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(synth_volume_layout())
                    .with_spacer(10.0)
                    .with_child(env_layout("Env1", SynthUIData::env1))
                    .with_spacer(10.0)
                    .with_child(env_layout("Env2", SynthUIData::env2));
    synth_ui.root.add_child(control_layout.padding((20.0, 0.0, 0.0, 0.0)));

    synth_ui.center().background(BACKGROUND_COLOR)
}
