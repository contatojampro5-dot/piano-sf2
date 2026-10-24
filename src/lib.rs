//! Piano SF2: programa leve para tocar SF2 ao vivo (igreja) com piano MIDI e mesinha SMC-Mixer.

pub mod blemidi;
pub mod core;
pub mod engine;
pub mod granular;
pub mod model;
pub mod pads;
pub mod teoria;

#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "app")]
pub mod audio;
#[cfg(all(feature = "app", windows))]
pub mod ble;
#[cfg(all(feature = "app", windows))]
pub mod btdirect;
#[cfg(feature = "app")]
pub mod midi;
#[cfg(feature = "app")]
pub mod theme;

#[cfg(test)]
mod testsf;
