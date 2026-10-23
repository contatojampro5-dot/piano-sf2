//! Piano SF2: programa leve para tocar SF2 ao vivo (igreja) com piano MIDI e mesinha SMC-Mixer.

pub mod core;
pub mod engine;
pub mod model;

#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "app")]
pub mod audio;
#[cfg(feature = "app")]
pub mod midi;

#[cfg(test)]
mod testsf;
