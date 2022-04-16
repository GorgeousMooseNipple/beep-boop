use crate::synth::WaveForm;
use super::widgets::WaveFormUI;


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