# spectral-music-v2

**Complete spectral music theory: chords as graph nodes, voice-leading as edges, tension as eigenvalue content, splines as voice-leading paths.**

Pure Rust. A library that treats music theory as spectral graph theory — because the math lines up. The conservation ratio of a ii-V-I progression outperforms random chord sequences, and Fibonacci-spaced progressions converge toward the golden ratio.

## What This Gives You

- **Chord theory** — Major, Minor, Dominant, Diminished, Half-Diminished, Augmented, Suspended with spectral tension
- **Voice leading** — Minimal-movement assignments with curvature analysis
- **Progressions as graphs** — Laplacian eigenvalues encode harmonic tension, CR measures consonance
- **Spline voice leading** — Each voice is a spline through chord tones; smoothness = quality
- **Fibonacci harmony** — Golden ratio tension/resolution, Fibonacci-spaced progressions
- **Standard progressions** — ii-V-I, I-vi-ii-V, circle of fifths, Coltrane changes, all ranked by CR

## The Core Idea

A chord progression is a weighted graph. Chords are nodes. Voice-leading distances (total semitone movement between chords) are edge weights. The Laplacian eigenvalues encode the harmonic tension profile. The conservation ratio CR = λ₂/λ_max measures how "conserved" the voice-leading energy is — common-practice progressions like ii-V-I score higher than random sequences.

## Quick Start

```rust
use spectral_music_v2::chord::Chord;
use spectral_music_v2::progression::{two_five_one, circle_of_fifths};
use spectral_music_v2::fibonacci_harmony::FibonacciHarmony;

// Build chords
let dm7 = Chord::from_name("Dm7").unwrap();
let g7 = Chord::from_name("G7").unwrap();
let cmaj7 = Chord::from_name("Cmaj7").unwrap();

// Chord tension: Diminished > Dominant > Major
assert!(Chord::from_name("Cdim").unwrap().tension() > g7.tension());
assert!(g7.tension() > cmaj7.tension());

// The ii-V-I — most conserved progression in jazz
let prog = two_five_one();
let cr = prog.conservation();
let resolution = prog.resolution_quality();

// Fibonacci-spaced chord progression
let fib_prog = FibonacciHarmony::fibonacci_progression(0, 6);

// Rank all standard jazz progressions by CR
let ranked = FibonacciHarmony::jazz_progressions_by_cr();
for (name, cr) in &ranked {
    println!("{}: CR = {:.4}", name, cr);
}

// Golden ratio check: does this progression's CR approach 1/φ?
assert!(FibonacciHarmony::approaches_golden_ratio(1.0 / FibonacciHarmony::phi()));
```

## API Reference

### Chord

| Method | Description |
|--------|-------------|
| `Chord::from_name("Dm7")` | Parse from standard notation |
| `.tension()` | Spectral tension ∈ [0, 1] |
| `.consonance()` | 1 - tension |
| `.pitches()` | MIDI pitch values |
| `.voice_leading_cost(&other)` | Minimal semitone movement |

### Progression

| Method | Description |
|--------|-------------|
| `Progression::from_chord_sequence(chords)` | Chain graph of sequential pairs |
| `.conservation()` | CR of the voice-leading graph |
| `.tension_profile()` | Tension at each chord |
| `.resolution_quality()` | How well tension resolves (0-1) |
| `.fiedler_vector()` | Natural harmonic partition |
| `.avg_smoothness()` | Inverse of average voice-leading cost |

### SplineVoiceLeading

| Method | Description |
|--------|-------------|
| `SplineVoiceLeading::smoothest(chords, n_voices)` | Minimal-movement voice assignment |
| `.curvature()` | Second-difference curvature per voice |
| `.conservation()` | Overall smoothness measure |
| `.total_cost()` | Total semitone movement |

### FibonacciHarmony

| Method | Description |
|--------|-------------|
| `.fibonacci_progression(key, gens)` | Circle-of-fifths at Fibonacci intervals |
| `.golden_tension_resolution(&prog)` | Peak/final tension ratio vs φ |
| `.jazz_progressions_by_cr()` | All standard progressions ranked by CR |
| `.phi()` | Golden ratio constant |

## How It Fits

Part of the SuperInstance spectral ecosystem:

- **[spectral-graph-core](https://github.com/SuperInstance/spectral-graph-core)** — Laplacian eigenvalues and CR
- **spectral-music-v2** — Music as spectral graph theory (this repo)
- **[spline-instrument](https://github.com/SuperInstance/spline-instrument)** — Hear the waveforms
- **[topology-lab](https://github.com/SuperInstance/topology-lab)** — Interactive Music Explorer lab

## Testing

```bash
cargo test
```

## Installation

```toml
[dependencies]
spectral-music-v2 = { git = "https://github.com/SuperInstance/spectral-music-v2" }
```

## License

MIT

Part of the [SuperInstance](https://github.com/SuperInstance) ecosystem.
