// spectral-music-v2: Complete spectral music theory engine
// Music IS spectral graph theory. Notes are nodes, voice-leading distance is edge weight.
// The tension graph Laplacian's eigenvalues ARE the harmonic content.

pub mod pitch;
pub mod chord;
pub mod progression;
pub mod spline_voice_leading;
pub mod fibonacci_harmony;

pub use pitch::{Pitch, Interval};
pub use chord::{Chord, ChordQuality};
pub use progression::Progression;
pub use spline_voice_leading::{SplineVoiceLeading, VoiceEvent};
pub use fibonacci_harmony::FibonacciHarmony;
