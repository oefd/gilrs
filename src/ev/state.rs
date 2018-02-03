// Copyright 2017 Mateusz Sieczko and other GilRs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use ev::Code;

use fnv::FnvHashMap;

use std::collections::hash_map;
use std::iter::Iterator;
use std::time::SystemTime;

/// Cached gamepad state.
#[derive(Clone, Debug)]
pub struct GamepadState {
    // Indexed by EvCode (nec)
    buttons: FnvHashMap<Code, ButtonData>,
    // Indexed by EvCode (nec)
    axes: FnvHashMap<Code, AxisData>,
}

impl GamepadState {
    pub(crate) fn new() -> Self {
        GamepadState {
            buttons: FnvHashMap::default(),
            axes: FnvHashMap::default(),
        }
    }

    /// Returns `true` if given button is pressed. Returns `false` if there is no information about
    /// `btn` or it is not pressed.
    pub fn is_pressed(&self, btn: &Code) -> bool {
        self.buttons
            .get(btn)
            .map(|s| s.is_pressed())
            .unwrap_or(false)
    }

    /// Returns value of axis or 0.0 when there is no information about axis.
    pub fn value(&self, axis: &Code) -> f32 {
        self.axes.get(axis).map(|s| s.value()).unwrap_or(0.0)
    }

    /// Iterate over buttons data.
    pub fn buttons(&self) -> ButtonDataIter {
        ButtonDataIter(self.buttons.iter())
    }

    /// Iterate over axes data.
    pub fn axes(&self) -> AxisDataIter {
        AxisDataIter(self.axes.iter())
    }

    /// Returns button state and when it changed.
    pub fn button_data(&self, btn: &Code) -> Option<&ButtonData> {
        self.buttons.get(btn)
    }

    /// Returns axis state and when it changed.
    pub fn axis_data(&self, axis: &Code) -> Option<&AxisData> {
        self.axes.get(axis)
    }

    pub(crate) fn set_btn_pressed(
        &mut self,
        btn: Code,
        pressed: bool,
        counter: u64,
        timestamp: SystemTime,
    ) {
        let data = self.buttons.entry(btn).or_insert_with(|| {
            ButtonData::new(
                if pressed { 1.0 } else { 0.0 },
                pressed,
                false,
                counter,
                timestamp,
            )
        });
        data.is_pressed = pressed;
        data.is_repeating = false;
        data.counter = counter;
        data.last_event_ts = timestamp;
    }

    pub(crate) fn set_btn_repeating(&mut self, btn: Code, counter: u64, timestamp: SystemTime) {
        let data = self.buttons
            .entry(btn)
            .or_insert_with(|| ButtonData::new(1.0, true, true, counter, timestamp));
        data.is_repeating = true;
        data.counter = counter;
        data.last_event_ts = timestamp;
    }

    pub(crate) fn set_btn_value(
        &mut self,
        btn: Code,
        value: f32,
        counter: u64,
        timestamp: SystemTime,
    ) {
        let data = self.buttons
            .entry(btn)
            .or_insert_with(|| ButtonData::new(value, false, false, counter, timestamp));
        data.value = value;
        data.counter = counter;
        data.last_event_ts = timestamp;
    }

    pub(crate) fn update_axis(&mut self, axis: Code, data: AxisData) {
        self.axes.insert(axis, data);
    }
}

/// Iterator over `ButtonData`.
pub struct ButtonDataIter<'a>(hash_map::Iter<'a, Code, ButtonData>);

/// Iterator over `AxisData`.
pub struct AxisDataIter<'a>(hash_map::Iter<'a, Code, AxisData>);

impl<'a> Iterator for ButtonDataIter<'a> {
    type Item = (Code, &'a ButtonData);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (*k, v))
    }
}

impl<'a> Iterator for AxisDataIter<'a> {
    type Item = (Code, &'a AxisData);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (*k, v))
    }
}

/// Information about button stored in `State`.
#[derive(Clone, Copy, Debug)]
pub struct ButtonData {
    last_event_ts: SystemTime,
    counter: u64,
    value: f32,
    is_pressed: bool,
    is_repeating: bool,
}

impl ButtonData {
    pub(crate) fn new(
        value: f32,
        pressed: bool,
        repeating: bool,
        counter: u64,
        time: SystemTime,
    ) -> Self {
        ButtonData {
            last_event_ts: time,
            counter,
            value,
            is_pressed: pressed,
            is_repeating: repeating,
        }
    }

    /// Returns `true` if button is pressed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns value of button.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns `true` if button is repeating.
    pub fn is_repeating(&self) -> bool {
        self.is_repeating
    }

    /// Returns value of counter when button state last changed.
    pub fn counter(&self) -> u64 {
        self.counter
    }

    /// Returns when button state last changed.
    pub fn timestamp(&self) -> SystemTime {
        self.last_event_ts
    }
}

/// Information about axis stored in `State`.
#[derive(Clone, Copy, Debug)]
pub struct AxisData {
    last_event_ts: SystemTime,
    last_event_c: u64,
    value: f32,
}

impl AxisData {
    pub(crate) fn new(value: f32, counter: u64, time: SystemTime) -> Self {
        AxisData {
            last_event_ts: time,
            last_event_c: counter,
            value,
        }
    }
    /// Returns value of axis.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns value of counter when axis value last changed.
    pub fn counter(&self) -> u64 {
        self.last_event_c
    }

    /// Returns when axis value last changed.
    pub fn timestamp(&self) -> SystemTime {
        self.last_event_ts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn button_data() {
        // Who needs rand crate
        let mut state = 1234567890123456789u64;
        let mut xorshift = || {
            let mut x = state;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            state = x;

            x.wrapping_mul(0x2545F4914F6CDD1D)
        };

        for _ in 0..(1024 * 1024 * 16) {
            let counter = xorshift() & 0x3FFF_FFFF_FFFF_FFFF;
            let pressed = xorshift() % 2 == 1;
            let repeating = xorshift() % 2 == 1;
            let btn = ButtonData::new(pressed, repeating, counter, SystemTime::now());
            assert_eq!(btn.is_pressed(), pressed);
            assert_eq!(btn.is_repeating(), repeating);
            assert_eq!(btn.counter(), counter);
        }
    }
}
