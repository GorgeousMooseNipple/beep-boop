use super::Released;
use std::ops::{Add, Sub};
use std::time::Instant;

type Milliseconds = u32;

#[allow(dead_code)]
pub mod adsr_constraints {
    pub const MIN_ATTACK: f32 = 1.;
    pub const MAX_ATTACK: f32 = 3000.;
    pub const MIN_DECAY: f32 = 1.;
    pub const MAX_DECAY: f32 = 3000.;
    pub const MIN_SUSTAIN: f32 = 0.;
    pub const MAX_SUSTAIN: f32 = 1.;
    pub const MIN_RELEASE: f32 = 1.;
    pub const MAX_RELEASE: f32 = 3000.;
}

pub enum ADSRParam {
    Attack(f32),
    Decay(f32),
    Sustain(f32),
    Release(f32),
}

#[derive(Clone)]
pub struct ADSR {
    sample_rate: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    attack_incr: f32,
    decay_decr: f32,
    #[allow(dead_code)]
    release_decr: f32,
    release_samples: f32,
}

impl ADSR {
    pub fn new(
        sample_rate: f32,
        attack: Milliseconds,
        decay: Milliseconds,
        sustain: f32,
        release: Milliseconds,
    ) -> Self {
        let attack = adsr_constraints::MIN_ATTACK.max(attack as f32);
        let decay = adsr_constraints::MIN_DECAY.max(decay as f32);
        let release = adsr_constraints::MIN_RELEASE.max(release as f32);
        let attack_incr = 1.0 / (attack / 1000.0 * sample_rate);
        let decay_decr = -((1.0 - sustain) / (decay / 1000.0 * sample_rate));
        let release_decr = -(sustain / (release / 1000.0 * sample_rate));
        let release_samples = release / 1000.0 * sample_rate;
        Self {
            sample_rate,
            attack,
            decay,
            sustain,
            release,
            attack_incr,
            decay_decr,
            release_decr,
            release_samples,
        }
    }

    pub fn set_parameter(&mut self, param: ADSRParam) {
        match param {
            ADSRParam::Attack(val) => {
                self.attack = val.max(1.0);
                self.attack_incr = 1.0 / (self.attack / 1000.0 * self.sample_rate);
            }
            ADSRParam::Decay(val) => {
                self.decay = val.max(3.0);
                self.decay_decr = -((1.0 - self.sustain) / (self.decay / 1000.0 * self.sample_rate));
            }
            ADSRParam::Sustain(val) => {
                self.sustain = val;
                // Update decay decrement too, because it depends on sustain value
                self.decay_decr = -((1.0 - self.sustain) / (self.decay / 1000.0 * self.sample_rate));
            }
            ADSRParam::Release(val) => {
                self.release = val;
                self.release_samples = self.release / 1000.0 * self.sample_rate;
            }
        }
    }

    // Incremental version
    pub fn get_volume_incr(
        &self,
        current: &f32,
        triggered: &Instant,
        released: &Option<Released>,
    ) -> f32 {
        if let Some(r) = released {
            // Release stage
            current - (r.value / self.release_samples)
        } else {
            let alive_for = triggered.elapsed().as_millis() as f32;
            // Attack stage
            if alive_for <= self.attack {
                return current + self.attack_incr;
            }
            // Decay stage
            if alive_for <= self.attack.add(self.decay) {
                let output = current + self.decay_decr;
                if output > self.sustain {
                    return output;
                }
            }
            self.sustain
        }
    }

    // Old heavy version
    #[allow(dead_code)]
    pub fn get_volume(&self, triggered: &Instant, released: &Option<Released>) -> f32 {
        match released {
            Some(ref released) => {
                let released_for = released.time.elapsed().as_millis() as f32;
                return released.value * (1.0 - released_for / self.release);
            }
            None => {
                let active_for = triggered.elapsed().as_millis() as f32;
                if active_for <= self.attack {
                    return active_for / self.attack;
                }
                if active_for <= self.attack.add(self.decay) {
                    let to_sustain = 1.0 - self.sustain;
                    let cur_fraction = 1.0 - active_for.sub(self.attack) / self.decay;
                    return self.sustain + to_sustain * cur_fraction;
                }
                return self.sustain;
            }
        }
    }
}
