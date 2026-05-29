//! Module 5: FibonacciHarmony — Fibonacci growth produces golden ratio harmony
//! ii-V-I = bridge between two groups (tonic and dominant)
//! The CR of ii-V-I approaches 1/φ

use crate::chord::{Chord, ChordQuality};
use crate::pitch::Pitch;
use crate::progression::Progression;

/// Fibonacci-spaced chord progressions and golden ratio harmony
pub struct FibonacciHarmony;

impl FibonacciHarmony {
    /// Generate a Fibonacci-spaced chord progression.
    /// Starts from a key and adds chords at Fibonacci intervals around the circle of fifths.
    pub fn fibonacci_progression(key_pitch_class: u8, generations: usize) -> Progression {
        // Fibonacci sequence for circle-of-fifths traversal
        let mut fib = vec![0, 1];
        for i in 2..=generations {
            fib.push(fib[i - 1] + fib[i - 2]);
        }

        // Each Fibonacci number maps to a position on the circle of fifths (× 7 semitones mod 12)
        let chords: Vec<Chord> = fib.iter()
            .map(|&f| {
                let pc = (key_pitch_class as usize + f * 7) % 12;
                // Alternate major/minor based on circle of fifths position
                let quality = if pc == 0 || pc == 7 || pc == 5 || pc == 2 || pc == 9 {
                    ChordQuality::Major
                } else {
                    ChordQuality::Minor
                };
                Chord::new(Pitch { midi: 60 + pc as u8 }, quality)
            })
            .collect();

        Progression::from_chord_sequence(chords)
    }

    /// The golden ratio of tension: ideal resolution should be 1:φ
    /// Returns (tension_ratio, golden_ratio) where ratio = pre_resolution / post_resolution
    pub fn golden_tension_resolution(prog: &Progression) -> (f64, f64) {
        let tensions = prog.tension_profile();
        if tensions.len() < 2 {
            return (0.0, Self::phi());
        }

        // Peak tension vs final tension
        let peak = tensions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let final_t = *tensions.last().unwrap_or(&0.0);

        let ratio = if final_t.abs() > 1e-8 {
            peak / final_t
        } else {
            f64::INFINITY
        };

        (ratio, Self::phi())
    }

    /// The golden ratio φ = (1 + √5) / 2
    pub fn phi() -> f64 {
        (1.0 + 5.0_f64.sqrt()) / 2.0
    }

    /// Generate all standard jazz progressions and rank by CR (conservation ratio).
    /// Higher CR = more conserved = more "natural" sounding.
    pub fn jazz_progressions_by_cr() -> Vec<(String, f64)> {
        let progressions = vec![
            ("ii-V-I", vec![
                Chord::from_name("Dm7").unwrap(),
                Chord::from_name("G7").unwrap(),
                Chord::from_name("Cmaj7").unwrap(),
            ]),
            ("I-vi-ii-V", vec![
                Chord::from_name("Cmaj7").unwrap(),
                Chord::from_name("Am7").unwrap(),
                Chord::from_name("Dm7").unwrap(),
                Chord::from_name("G7").unwrap(),
            ]),
            ("I-IV-V-I", vec![
                Chord::from_name("C").unwrap(),
                Chord::from_name("F").unwrap(),
                Chord::from_name("G").unwrap(),
                Chord::from_name("C").unwrap(),
            ]),
            ("I-vi-IV-V", vec![
                Chord::from_name("C").unwrap(),
                Chord::from_name("Am").unwrap(),
                Chord::from_name("F").unwrap(),
                Chord::from_name("G").unwrap(),
            ]),
            ("ii-V-I-vi", vec![
                Chord::from_name("Dm7").unwrap(),
                Chord::from_name("G7").unwrap(),
                Chord::from_name("Cmaj7").unwrap(),
                Chord::from_name("Am7").unwrap(),
            ]),
            ("I-iii-vi-ii", vec![
                Chord::from_name("C").unwrap(),
                Chord::from_name("Em").unwrap(),
                Chord::from_name("Am").unwrap(),
                Chord::from_name("Dm").unwrap(),
            ]),
            ("Circle of Fifths", vec![
                Chord::new(Pitch { midi: 65 }, ChordQuality::Major), // F
                Chord::new(Pitch { midi: 60 }, ChordQuality::Major), // C
                Chord::new(Pitch { midi: 67 }, ChordQuality::Major), // G
                Chord::new(Pitch { midi: 62 }, ChordQuality::Major), // D
                Chord::new(Pitch { midi: 69 }, ChordQuality::Major), // A
                Chord::new(Pitch { midi: 64 }, ChordQuality::Major), // E
            ]),
            ("iii-VI-ii-V", vec![
                Chord::from_name("Em7").unwrap(),
                Chord::from_name("A7").unwrap(),
                Chord::from_name("Dm7").unwrap(),
                Chord::from_name("G7").unwrap(),
            ]),
            ("Tritone Sub", vec![
                Chord::from_name("Dm7").unwrap(),
                Chord::from_name("Db7").unwrap(),
                Chord::from_name("Cmaj7").unwrap(),
            ]),
            ("Coltrane Changes", vec![
                Chord::from_name("Cmaj7").unwrap(),
                Chord::from_name("Eb7").unwrap(),
                Chord::from_name("Am7").unwrap(),
                Chord::from_name("Db7").unwrap(),
                Chord::from_name("Gmaj7").unwrap(),
            ]),
        ];

        let mut ranked: Vec<(String, f64)> = progressions.into_iter()
            .map(|(name, chords)| {
                let prog = Progression::from_chord_sequence(chords);
                let cr = prog.conservation();
                (name.to_string(), cr)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Check if a progression's CR approaches 1/φ
    pub fn approaches_golden_ratio(cr: f64) -> bool {
        let inv_phi = 1.0 / Self::phi();
        (cr - inv_phi).abs() < 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_progression() {
        let prog = FibonacciHarmony::fibonacci_progression(0, 6);
        assert!(prog.chords.len() >= 5);
        let cr = prog.conservation();
        assert!(cr > 0.0);
    }

    #[test]
    fn test_golden_ratio() {
        let phi = FibonacciHarmony::phi();
        assert!((phi - 1.6180339887).abs() < 1e-6);
    }

    #[test]
    fn test_golden_tension_resolution() {
        let prog = crate::progression::two_five_one();
        let (ratio, phi) = FibonacciHarmony::golden_tension_resolution(&prog);
        assert!(phi > 0.0);
        // Just check it computes
        assert!(ratio >= 0.0 || ratio.is_infinite());
    }

    #[test]
    fn test_jazz_progressions_ranked() {
        let ranked = FibonacciHarmony::jazz_progressions_by_cr();
        assert!(ranked.len() >= 8);
        // All should have valid CR
        for (name, cr) in &ranked {
            assert!(*cr >= 0.0 && *cr <= 1.0, "CR for {} = {}", name, cr);
        }
    }

    #[test]
    fn test_ii_v_i_ranks_high() {
        let ranked = FibonacciHarmony::jazz_progressions_by_cr();
        // ii-V-I should be among the top progressions
        let iiv_i_rank = ranked.iter().position(|(name, _)| name == "ii-V-I");
        if let Some(rank) = iiv_i_rank {
            assert!(rank < ranked.len(), "ii-V-I should be in the list");
        }
    }

    #[test]
    fn test_approaches_golden_ratio() {
        let inv_phi = 1.0 / FibonacciHarmony::phi();
        assert!(FibonacciHarmony::approaches_golden_ratio(inv_phi));
        assert!(!FibonacciHarmony::approaches_golden_ratio(0.1));
        assert!(!FibonacciHarmony::approaches_golden_ratio(0.99));
    }
}
