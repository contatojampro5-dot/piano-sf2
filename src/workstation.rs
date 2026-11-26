use std::sync::atomic::{AtomicU32, Ordering};

use crate::efeitos::{step_samples, BPM_MIN};

pub const SLOTS: usize = 4;
const MAX_BARS: u8 = 8;
const MAX_RATE: u32 = 48_000;

#[derive(Clone, Copy)]
pub enum Action {
    Record(usize),
    Play(usize),
    Stop(usize),
    Clear(usize),
    Volume(usize, f32),
    Mute(usize, bool),
    StopAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum State {
    Empty,
    Armed,
    Recording,
    Finishing,
    Ready,
    Queued,
    Playing,
}

#[derive(Default)]
pub struct Status {
    slots: [AtomicU32; SLOTS],
    rate: AtomicU32,
}

impl Status {
    pub fn available(&self) -> bool {
        matches!(self.rate.load(Ordering::Relaxed), 1..=MAX_RATE)
    }

    pub fn slot(&self, index: usize) -> (State, f32, bool) {
        let bits = self.slots[index].load(Ordering::Relaxed);
        let state = match bits & 7 {
            1 => State::Armed,
            2 => State::Recording,
            3 => State::Finishing,
            4 => State::Ready,
            5 => State::Queued,
            6 => State::Playing,
            _ => State::Empty,
        };
        // Zero is the default atomic value, representing unity volume.
        let volume = 1.0 - ((bits >> 4) as f32 / 65535.0);
        (state, volume, bits & 8 != 0)
    }
}

struct Slot {
    samples: Box<[[f32; 2]]>,
    len: usize,
    position: usize,
    bars: u8,
    state: State,
    volume: f32,
    muted: bool,
}

impl Slot {
    fn new(capacity: usize) -> Self {
        Self {
            samples: vec![[0.0; 2]; capacity].into_boxed_slice(),
            len: 0,
            position: 0,
            bars: 0,
            state: State::Empty,
            volume: 1.0,
            muted: false,
        }
    }

    fn clear(&mut self) {
        self.len = 0;
        self.state = State::Empty;
    }

    fn stop(&mut self) {
        match self.state {
            State::Armed | State::Recording | State::Finishing => self.clear(),
            State::Queued | State::Playing => self.state = State::Ready,
            _ => {}
        }
    }
}

pub(crate) struct Looper {
    slots: [Slot; SLOTS],
    rate: u32,
    next_bar: f64,
    bar_samples: f64,
}

impl Looper {
    pub(crate) fn new() -> Self {
        let capacity =
            (step_samples(BPM_MIN, 1, MAX_RATE) * 4.0 * f64::from(MAX_BARS)).ceil() as usize;
        Self::with_capacity(capacity)
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: std::array::from_fn(|_| Slot::new(capacity)),
            rate: 0,
            next_bar: 0.0,
            bar_samples: 0.0,
        }
    }

    pub(crate) fn sync_rate(&mut self, rate: u32, clock: u64) {
        if rate != self.rate {
            self.rate = rate;
            self.next_bar = clock as f64;
            for slot in &mut self.slots {
                slot.clear();
            }
        }
    }

    pub(crate) fn sync_tempo(&mut self, clock: u64) {
        if !self
            .slots
            .iter()
            .any(|slot| matches!(slot.state, State::Recording | State::Finishing))
        {
            self.next_bar = clock as f64;
        }
    }

    pub(crate) fn action(&mut self, action: Action) {
        match action {
            Action::Record(index) if (1..=MAX_RATE).contains(&self.rate) => {
                let slot = &mut self.slots[index];
                slot.state = match slot.state {
                    State::Empty => State::Armed,
                    State::Armed => State::Empty,
                    State::Recording => State::Finishing,
                    State::Finishing => State::Recording,
                    state => state,
                };
            }
            Action::Play(index) => {
                let slot = &mut self.slots[index];
                if slot.state == State::Ready {
                    slot.state = State::Queued;
                }
            }
            Action::Stop(index) => self.slots[index].stop(),
            Action::Clear(index) => self.slots[index].clear(),
            Action::Volume(index, volume) => self.slots[index].volume = volume.clamp(0.0, 1.0),
            Action::Mute(index, muted) => self.slots[index].muted = muted,
            Action::StopAll => self.slots.iter_mut().for_each(Slot::stop),
            Action::Record(_) => {}
        }
    }

    fn advance_bar(&mut self, bpm: f32) {
        let mut keep_tempo = false;
        for slot in &mut self.slots {
            match slot.state {
                State::Armed => {
                    slot.len = 0;
                    slot.bars = 0;
                    slot.state = State::Recording;
                }
                State::Recording | State::Finishing => {
                    slot.bars += 1;
                    if slot.state == State::Finishing || slot.bars == MAX_BARS {
                        slot.state = State::Playing;
                        slot.position = 0;
                    } else {
                        keep_tempo = true;
                    }
                }
                State::Queued => {
                    slot.position = 0;
                    slot.state = State::Playing;
                }
                _ => {}
            }
        }
        if !keep_tempo {
            self.bar_samples = 4.0 * step_samples(bpm, 1, self.rate);
        }
        self.next_bar += self.bar_samples;
    }

    pub(crate) fn mix(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        clock: u64,
        bpm: f32,
        master: f32,
    ) {
        if !(1..=MAX_RATE).contains(&self.rate) {
            return;
        }
        let end = clock + left.len() as u64;
        let mut frame = clock;
        while frame < end {
            if frame >= self.next_bar.ceil() as u64 {
                self.advance_bar(bpm);
            }
            let limit = end.min(self.next_bar.ceil() as u64);
            let range = (frame - clock) as usize..(limit - clock) as usize;
            let l = &mut left[range.clone()];
            let r = &mut right[range];
            // All captures see the same live mix, before any return enters it.
            // Undo the live mix's master gain so the return follows the current fader once.
            let capture_gain = if master > 0.0 { master.recip() } else { 1.0 };
            for slot in &mut self.slots {
                if matches!(slot.state, State::Recording | State::Finishing) {
                    let end = slot.len + l.len();
                    if let Some(samples) = slot.samples.get_mut(slot.len..end) {
                        for ((sample, &a), &b) in samples.iter_mut().zip(l.iter()).zip(r.iter()) {
                            *sample = [a * capture_gain, b * capture_gain];
                        }
                        slot.len = end;
                    } else {
                        slot.clear();
                    }
                }
            }
            for slot in &mut self.slots {
                if slot.state == State::Playing {
                    let gain = slot.volume * master;
                    for (a, b) in l.iter_mut().zip(r.iter_mut()) {
                        let [x, y] = slot.samples[slot.position];
                        if !slot.muted {
                            *a += x * gain;
                            *b += y * gain;
                        }
                        slot.position += 1;
                        if slot.position == slot.len {
                            slot.position = 0;
                        }
                    }
                }
            }
            frame = limit;
        }
    }

    pub(crate) fn publish(&self, status: &Status) {
        status.rate.store(self.rate, Ordering::Relaxed);
        for (slot, published) in self.slots.iter().zip(&status.slots) {
            let volume = ((1.0 - slot.volume) * 65535.0).round() as u32;
            let bits = (volume << 4) | (u32::from(slot.muted) << 3) | slot.state as u32;
            published.store(bits, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // At 60 BPM and 4 Hz, one 4/4 bar is 16 frames.
    fn render(looper: &mut Looper, clock: u64, frames: usize, value: f32, bpm: f32) -> Vec<f32> {
        let mut left = vec![value; frames];
        let mut right = vec![-value; frames];
        looper.mix(&mut left, &mut right, clock, bpm, 1.0);
        for (&l, &r) in left.iter().zip(&right) {
            assert_eq!(l, -r);
        }
        left
    }

    #[test]
    fn four_slots_quantize_and_keep_tempo_until_the_take_ends() {
        let mut looper = Looper::with_capacity(128);
        looper.sync_rate(4, 0);
        render(&mut looper, 0, 14, 0.0, 60.0);
        for index in 0..SLOTS {
            looper.action(Action::Record(index));
        }
        render(&mut looper, 14, 5, 0.25, 60.0);
        for slot in &looper.slots {
            assert_eq!((slot.state, slot.len), (State::Recording, 3));
        }
        for index in 0..SLOTS {
            looper.action(Action::Record(index));
        }
        let audio = render(&mut looper, 19, 15, 0.0, 120.0);
        assert_eq!(&audio[13..], &[1.0, 1.0]);
        for slot in &looper.slots {
            assert_eq!((slot.state, slot.len), (State::Playing, 16));
        }
        looper.action(Action::Volume(0, 0.5));
        looper.action(Action::Mute(1, true));
        looper.action(Action::Stop(2));
        looper.action(Action::Clear(3));
        assert_eq!(render(&mut looper, 34, 17, 0.0, 120.0)[16], 0.125);
        looper.action(Action::Play(2));
        looper.action(Action::Record(3));
        looper.action(Action::StopAll);
        assert_eq!(looper.slots[2].state, State::Ready);
        assert_eq!(looper.slots[3].state, State::Empty);
        looper.sync_rate(8, 51);
        assert!(looper.slots.iter().all(|slot| slot.state == State::Empty));
    }

    #[test]
    fn recording_stops_at_capacity_and_small_buffers_fail_safely() {
        for capacity in [127, 128] {
            let mut looper = Looper::with_capacity(capacity);
            looper.sync_rate(4, 0);
            looper.action(Action::Record(0));
            render(&mut looper, 0, 129, 0.0, 60.0);
            let slot = &looper.slots[0];
            if capacity == 128 {
                assert_eq!((slot.state, slot.len), (State::Playing, 128));
            } else {
                assert_eq!((slot.state, slot.len), (State::Empty, 0));
            }
        }
    }
}
