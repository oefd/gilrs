use gamepad::{Button, Gamepad};
use std::u16::MAX as U16_MAX;
use platform;
use GamepadExt;

#[derive(Debug)]
pub struct Effect<'a> {
    inner: platform::Effect<'a>,
}

impl<'a> Effect<'a> {
    pub fn new(gamepad: &'a Gamepad, data: EffectData) -> Option<Self> {
        platform::Effect::new(gamepad.inner(), data).map(|effect| Effect { inner: effect })
    }

    pub fn upload(&mut self, data: EffectData) -> Option<()> {
        self.inner.upload(data)
    }

    pub fn play(&mut self, n: u16) {
        self.inner.play(n)
    }

    pub fn stop(&mut self) {
        self.inner.stop()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct EffectData {
    pub wave: Waveform,
    pub direction: Direction,
    pub period: u16,
    pub magnitude: i16,
    pub offset: i16,
    pub phase: u16,
    pub envelope: Envelope,
    pub replay: Replay,
    pub trigger: Trigger,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Waveform {
    Square,
    Triangle,
    Sine,
}

impl Default for Waveform {
    fn default() -> Self { Waveform::Sine }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Direction {
    pub angle: u16,
}

impl From<f32> for Direction {
    fn from(f: f32) -> Self {
        let f = if f < 0.0 {
            0.0
        } else if f > 1.0 {
            1.0
        } else {
            f
        };
        Direction { angle: (U16_MAX as f32 * f) as u16 }
    }
}

impl From<[f32; 2]> for Direction {
    fn from(f: [f32; 2]) -> Self {
        (f[0].sin() + f[1].cos()).into()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Envelope {
    pub attack_length: u16,
    pub attack_level: u16,
    pub fade_length: u16,
    pub fade_level: u16,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Replay {
    pub length: u16,
    pub delay: u16,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Trigger {
    pub button: Button,
    pub interval: u16,
}
