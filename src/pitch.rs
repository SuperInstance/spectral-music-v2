//! Module 1: Pitch — the foundation
//! Notes are nodes in spectral space. Intervals are edges weighted by consonance.

/// A musical pitch in MIDI space (0-127)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pitch {
    pub midi: u8,
}

/// An interval between two pitches in semitones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Interval {
    pub semitones: i8,
}

impl Pitch {
    /// Create a pitch from note name, octave, and accidental.
    /// Note names: A-G, octave 0-10, sharp flag for #.
    /// MIDI: C4 = 60, A4 = 69
    pub fn from_note(name: char, octave: u8, sharp: bool) -> Pitch {
        let base = match name.to_uppercase().next().unwrap() {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => 0,
        };
        // MIDI: C-1 = 0, C0 = 12, C4 = 60, A4 = 69
        let midi = ((octave as i32) + 1) * 12 + base + if sharp { 1 } else { 0 };
        Pitch { midi: midi.clamp(0, 127) as u8 }
    }

    /// Frequency in Hz. A4 (midi 69) = 440Hz
    pub fn frequency(&self) -> f64 {
        440.0 * 2.0_f64.powf((self.midi as f64 - 69.0) / 12.0)
    }

    /// Interval from self to other
    pub fn interval_to(&self, other: &Pitch) -> Interval {
        Interval {
            semitones: other.midi as i8 - self.midi as i8,
        }
    }

    /// Transpose by interval
    pub fn transpose(&self, interval: Interval) -> Pitch {
        Pitch {
            midi: (self.midi as i8 + interval.semitones).clamp(0, 127) as u8,
        }
    }

    /// Pitch class (0-11, where 0 = C)
    pub fn pitch_class(&self) -> u8 {
        self.midi % 12
    }

    /// Note name for display in scientific pitch notation
    /// (consistent with `from_note`: C4 = MIDI 60, A4 = MIDI 69).
    pub fn name(&self) -> String {
        let names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let pc = self.pitch_class() as usize;
        // Scientific pitch notation: octave = midi / 12 - 1
        // (MIDI 0 = C-1, MIDI 12 = C0, MIDI 60 = C4, MIDI 69 = A4)
        let octave = self.midi as i32 / 12 - 1;
        format!("{}{}", names[pc], octave)
    }
}

impl Interval {
    /// Consonance based on spectral smoothness of combined harmonic series.
    /// Unison = 1.0, tritone = ~0.0.
    /// Based on ratio of harmonic series overlap: simpler ratios = more consonant.
    pub fn consonance(&self) -> f64 {
        let abs = self.semitones.unsigned_abs() as u8;
        let abs = abs % 12; // octave equivalence
        match abs {
            0 => 1.0,       // unison/perfect octave
            1 => 0.15,      // minor second
            2 => 0.25,      // major second
            3 => 0.55,      // minor third
            4 => 0.65,      // major third
            5 => 0.80,      // perfect fourth
            6 => 0.05,      // tritone
            7 => 0.90,      // perfect fifth
            8 => 0.55,      // minor sixth
            9 => 0.60,      // major sixth
            10 => 0.30,     // minor seventh
            11 => 0.20,     // major seventh
            _ => 0.0,
        }
    }

    /// Voice leading distance: minimal semitone movement.
    /// This IS the edge weight in the spectral graph.
    pub fn voice_leading_distance(&self) -> f64 {
        self.semitones.unsigned_abs() as f64
    }

    /// Minimal voice leading distance (considers octave equivalence)
    pub fn minimal_distance(&self) -> f64 {
        let abs = self.semitones.unsigned_abs() as f64;
        let wrapped = abs % 12.0;
        wrapped.min(12.0 - wrapped)
    }

    /// Is this a perfect fifth (or its inversion, perfect fourth)?
    pub fn is_perfect_consonance(&self) -> bool {
        let abs = self.semitones.unsigned_abs() % 12;
        abs == 0 || abs == 7 || abs == 5
    }

    /// Interval name
    pub fn name(&self) -> &'static str {
        let abs = self.semitones.unsigned_abs() % 12;
        match abs {
            0 => "P1",
            1 => "m2",
            2 => "M2",
            3 => "m3",
            4 => "M3",
            5 => "P4",
            6 => "TT",
            7 => "P5",
            8 => "m6",
            9 => "M6",
            10 => "m7",
            11 => "M7",
            _ => "?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_frequency_a4() {
        let a4 = Pitch::from_note('A', 4, false);
        assert!((a4.frequency() - 440.0).abs() < 0.01);
        assert_eq!(a4.midi, 69);
    }

    #[test]
    fn test_pitch_frequency_c4() {
        let c4 = Pitch::from_note('C', 4, false);
        // C4 = MIDI 60, freq = 440 * 2^(-9/12) ≈ 261.63
        assert!((c4.frequency() - 261.63).abs() < 0.1);
        assert_eq!(c4.midi, 60);
    }

    #[test]
    fn test_pitch_from_note() {
        assert_eq!(Pitch::from_note('C', 4, false).midi, 60);
        assert_eq!(Pitch::from_note('C', 4, true).midi, 61);
        assert_eq!(Pitch::from_note('A', 4, false).midi, 69);
        assert_eq!(Pitch::from_note('G', 4, true).midi, 68);
    }

    #[test]
    fn test_interval_consonance_ordering() {
        let p5 = Interval { semitones: 7 };
        let m3 = Interval { semitones: 3 };
        let m2 = Interval { semitones: 1 };
        let tt = Interval { semitones: 6 };
        // P5 > M3 > tritone in consonance
        assert!(p5.consonance() > m3.consonance());
        assert!(m3.consonance() > m2.consonance());
        assert!(m2.consonance() > tt.consonance());
    }

    #[test]
    fn test_interval_voice_leading() {
        let i = Interval { semitones: 3 };
        assert_eq!(i.voice_leading_distance(), 3.0);
    }

    #[test]
    fn test_pitch_transpose() {
        let c4 = Pitch::from_note('C', 4, false);
        let p5 = Interval { semitones: 7 };
        let g4 = c4.transpose(p5);
        assert_eq!(g4.midi, 67);
    }

    #[test]
    fn test_pitch_class() {
        assert_eq!(Pitch { midi: 60 }.pitch_class(), 0); // C
        assert_eq!(Pitch { midi: 69 }.pitch_class(), 9); // A
    }

    #[test]
    fn test_pitch_name() {
        // Scientific pitch notation: C4 = MIDI 60, A4 = MIDI 69
        assert_eq!(Pitch { midi: 60 }.name(), "C4");
        assert_eq!(Pitch { midi: 69 }.name(), "A4");
        assert_eq!(Pitch { midi: 12 }.name(), "C0");
        assert_eq!(Pitch { midi: 0 }.name(), "C-1");
        // Sharps
        assert_eq!(Pitch { midi: 61 }.name(), "C#4");
        assert_eq!(Pitch { midi: 66 }.name(), "F#4");
    }

    #[test]
    fn test_from_note_name_roundtrip() {
        // from_note and name() must use the same octave convention
        let a4 = Pitch::from_note('A', 4, false);
        assert_eq!(a4.name(), "A4");
        let c4 = Pitch::from_note('C', 4, false);
        assert_eq!(c4.name(), "C4");
        let g_sharp = Pitch::from_note('G', 4, true);
        assert_eq!(g_sharp.name(), "G#4");
    }
}
