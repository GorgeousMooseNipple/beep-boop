const TWO_PI: f32 = std::f32::consts::PI * 2.0;
const PI: f32 = std::f32::consts::PI;

#[derive(Clone, PartialEq)]
pub enum WaveForm {
    Sine,
    Square,
    Pulse25,
    Saw,
    Triangle,
}

impl WaveForm {
    pub fn get_wave(&self) -> Box<dyn Wave + Send> {
        match self {
            WaveForm::Sine => Box::new(Sine::new()),
            WaveForm::Square => Box::new(Square::new()),
            WaveForm::Pulse25 => Box::new(Pulse25::new()),
            WaveForm::Saw => Box::new(Saw::new()),
            WaveForm::Triangle => Box::new(Triangle::new()),
        }
    }
}


pub trait Wave {
    fn wave_func(&self, phase: f32) -> f32;
    fn next_phase(&self, phase: f32, incr: f32) -> f32;
    fn period(&self) -> f32;
}

pub struct Sine {
    period: f32,
}

impl Sine {
    pub fn new() -> Self {
        Self { period: TWO_PI }
    }
}

impl Wave for Sine {
    fn wave_func(&self, phase: f32) -> f32 {
        phase.sin()
    }

    fn next_phase(&self, mut phase: f32, incr: f32) -> f32 {
        phase += incr * self.period;
        if phase >= self.period {
            phase -= self.period
        }
        phase
    }

    fn period(&self) -> f32 {
        self.period
    }
}

pub struct Square {
    period: f32,
    half_period: f32,
}

impl Square {
    pub fn new() -> Self {
        Self {
            period: TWO_PI,
            half_period: PI,
        }
    }
}

impl Wave for Square {
    fn wave_func(&self, phase: f32) -> f32 {
        if phase <= self.half_period {
            0.7
        } else {
            -0.7
        }
    }

    fn next_phase(&self, mut phase: f32, incr: f32) -> f32 {
        phase += incr * self.period;
        if phase >= self.period {
            phase -= self.period;
        }
        phase
    }

    fn period(&self) -> f32 {
        self.period
    }
}

pub struct Saw {
    period: f32,
    half_period: f32,
}

impl Saw {
    pub fn new() -> Self {
        Self {
            period: 2.0,
            half_period: 1.0,
        }
    }
}

impl Wave for Saw {
    fn wave_func(&self, phase: f32) -> f32 {
        phase
    }

    fn next_phase(&self, mut phase: f32, incr: f32) -> f32 {
        phase += incr * self.period;
        if phase >= self.half_period {
            phase -= self.period;
        }
        phase
    }

    fn period(&self) -> f32 {
        self.period
    }
}

pub struct Pulse25 {
    period: f32,
    upper_part: f32,
}

impl Pulse25 {
    pub fn new() -> Self {
        Self {
            period: 2.0,
            upper_part: 2.0 * 0.25,
        }
    }
}

impl Wave for Pulse25 {
    fn wave_func(&self, phase: f32) -> f32 {
        if phase <= self.upper_part {
            0.7
        } else {
            -0.7
        }
    }

    fn next_phase(&self, mut phase: f32, incr: f32) -> f32 {
        phase += incr * self.period;
        if phase >= self.period {
            phase -= self.period;
        }
        phase
    }

    fn period(&self) -> f32 {
        self.period
    }
}

pub struct Triangle {
    period: f32,
    amplitide: f32,
    half_amplitude: f32,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            period: 4.0,
            amplitide: 2.0,
            half_amplitude: 1.0,
        }
    }
}

impl Wave for Triangle {
    fn wave_func(&self, phase: f32) -> f32 {
        -(phase - self.amplitide).abs() + self.half_amplitude
    }

    fn next_phase(&self, mut phase: f32, incr: f32) -> f32 {
        phase += incr * self.period;
        if phase >= self.period {
            phase -= self.period;
        }
        phase
    }

    fn period(&self) -> f32 {
        self.period
    }
}
