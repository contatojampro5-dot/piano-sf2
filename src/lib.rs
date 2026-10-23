//! Piano SF2: programa leve para tocar SF2 ao vivo (igreja) com piano MIDI e mesinha SMC-Mixer.

pub mod core;
pub mod engine;
pub mod model;
pub mod pads;

#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "app")]
pub mod audio;
#[cfg(feature = "app")]
pub mod midi;
#[cfg(feature = "app")]
pub mod theme;

#[cfg(test)]
mod testsf;
