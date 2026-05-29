//! Module 2: Chord — the node in the spectral graph
//! A chord is a set of pitches. Its tension = high-frequency eigenvalue content.

use crate::pitch::{Pitch, Interval};

/// Chord quality — the harmonic "flavor"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Dominant,
    Diminished,
    HalfDiminished,
    Augmented,
    Suspended,
}

impl ChordQuality {
    /// Interval pattern from root in semitones (excluding root)
    pub fn intervals(&self) -> &'static [i8] {
        match self {
            ChordQuality::Major => &[4, 7],
            ChordQuality::Minor => &[3, 7],
            ChordQuality::Dominant => &[4, 7, 10],
            ChordQuality::Diminished => &[3, 6, 9],
            ChordQuality::HalfDiminished => &[3, 6, 10],
            ChordQuality::Augmented => &[4, 8],
            ChordQuality::Suspended => &[5, 7],
        }
    }

    /// Base spectral tension of this quality
    /// More dissonant intervals → higher tension
    pub fn base_tension(&self) -> f64 {
        match self {
            ChordQuality::Major => 0.15,
            ChordQuality::Minor => 0.25,
            ChordQuality::Dominant => 0.50,
            ChordQuality::Diminished => 0.80,
            ChordQuality::HalfDiminished => 0.60,
            ChordQuality::Augmented => 0.55,
            ChordQuality::Suspended => 0.30,
        }
    }
}

/// A chord: root + quality + optional extensions
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    pub root: Pitch,
    pub quality: ChordQuality,
    pub extensions: Vec<u8>, // 7, 9, 11, 13 (already in intervals for 7th chords)
}

impl Chord {
    /// Create a new chord
    pub fn new(root: Pitch, quality: ChordQuality) -> Chord {
        Chord {
            root,
            quality,
            extensions: Vec::new(),
        }
    }

    /// Parse chord from name like "Dm7", "G7", "Cmaj7", "Bdim", "F#m7b5"
    pub fn from_name(name: &str) -> Option<Chord> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }

        // Parse root
        let (root_pc, rest) = {
            let c = name.chars().next().unwrap();
            let pc = match c.to_uppercase().next().unwrap() {
                'C' => 0, 'D' => 2, 'E' => 4, 'F' => 5,
                'G' => 7, 'A' => 9, 'B' => 11,
                _ => return None,
            };
            let rest = &name[1..];
            if rest.starts_with('#') || rest.starts_with('♯') {
                (pc + 1, &rest[1..])
            } else if rest.starts_with('b') || rest.starts_with('♭') {
                ((pc + 11) % 12, &rest[1..])
            } else {
                (pc, rest)
            }
        };

        // Default octave 4
        let root = Pitch { midi: 60 + root_pc };

        // Parse quality
        let (quality, extensions) = if rest.starts_with("maj7") {
            (ChordQuality::Major, vec![7])
        } else if rest.starts_with("maj") {
            (ChordQuality::Major, vec![])
        } else if rest.starts_with("m7b5") || rest.starts_with("ø7") {
            (ChordQuality::HalfDiminished, vec![7])
        } else if rest.starts_with("m7") || rest.starts_with("min7") {
            (ChordQuality::Minor, vec![7])
        } else if rest.starts_with("m") || rest.starts_with("min") {
            (ChordQuality::Minor, vec![])
        } else if rest.starts_with("dim7") {
            (ChordQuality::Diminished, vec![7])
        } else if rest.starts_with("dim") {
            (ChordQuality::Diminished, vec![])
        } else if rest.starts_with("aug") {
            (ChordQuality::Augmented, vec![])
        } else if rest.starts_with("sus") {
            (ChordQuality::Suspended, vec![])
        } else if rest.starts_with("7") {
            (ChordQuality::Dominant, vec![7])
        } else if rest.is_empty() {
            (ChordQuality::Major, vec![])
        } else {
            (ChordQuality::Major, vec![])
        };

        Some(Chord { root, quality, extensions })
    }

    /// All pitches in this chord
    pub fn pitches(&self) -> Vec<Pitch> {
        let mut result = vec![self.root];
        for &semitone in self.quality.intervals() {
            let pitch = Pitch {
                midi: self.root.midi + semitone as u8,
            };
            result.push(pitch);
        }
        result
    }

    /// Pitch classes (0-11) in this chord
    pub fn pitch_classes(&self) -> Vec<u8> {
        let mut pcs: Vec<u8> = self.pitches().iter().map(|p| p.pitch_class()).collect();
        pcs.sort();
        pcs.dedup();
        pcs
    }

    /// Spectral tension = base quality tension + extension tension + interval dissonance.
    /// Tension = high-frequency eigenvalue content of the chord's interval matrix.
    pub fn tension(&self) -> f64 {
        let mut tension = self.quality.base_tension();

        // Add tension from inter-pitch intervals (spectral content)
        let pitches = self.pitches();
        let mut total_dissonance = 0.0;
        let mut count = 0;
        for i in 0..pitches.len() {
            for j in (i + 1)..pitches.len() {
                let interval = pitches[i].interval_to(&pitches[j]);
                // Dissonance = 1 - consonance
                total_dissonance += 1.0 - interval.consonance();
                count += 1;
            }
        }
        if count > 0 {
            tension = tension * 0.5 + (total_dissonance / count as f64) * 0.5;
        }

        // Extensions add tension
        tension += self.extensions.len() as f64 * 0.05;

        tension.min(1.0)
    }

    /// Consonance = 1 - tension
    pub fn consonance(&self) -> f64 {
        1.0 - self.tension()
    }

    /// Voice leading intervals to another chord (minimal movement per voice)
    pub fn voice_leading_to(&self, other: &Chord) -> Vec<Interval> {
        let self_pitches = self.pitches();
        let other_pitches = other.pitches();
        let n = self_pitches.len().min(other_pitches.len());

        let mut intervals = Vec::with_capacity(n);
        for i in 0..n {
            intervals.push(self_pitches[i].interval_to(&other_pitches[i]));
        }
        intervals
    }

    /// Optimal voice leading cost (minimal total semitone movement).
    /// Uses greedy matching: for each pitch in self, find closest in other.
    pub fn voice_leading_cost(&self, other: &Chord) -> f64 {
        let self_pcs = self.pitch_classes();
        let other_pcs = other.pitch_classes();

        let n = self_pcs.len().max(other_pcs.len());
        if n == 0 {
            return 0.0;
        }

        // Pad with repeated pitch classes
        let mut from_pcs = self_pcs.clone();
        let mut to_pcs = other_pcs.clone();
        while from_pcs.len() < n {
            from_pcs.push(from_pcs[0]);
        }
        while to_pcs.len() < n {
            to_pcs.push(to_pcs[0]);
        }

        // Hungarian-like greedy: find minimal total distance
        // For simplicity, sort both and pair up
        from_pcs.sort();
        to_pcs.sort();

        let mut total = 0.0;
        for i in 0..n {
            let diff = (from_pcs[i] as i16 - to_pcs[i] as i16).unsigned_abs() as f64;
            let circular = diff.min(12.0 - diff);
            total += circular;
        }
        total
    }

    /// Name of the chord for display
    pub fn name(&self) -> String {
        let root_name = self.root.name();
        let quality_str = match self.quality {
            ChordQuality::Major => {
                if self.extensions.contains(&7) { "maj7" } else { "" }
            }
            ChordQuality::Minor => {
                if self.extensions.contains(&7) { "m7" } else { "m" }
            }
            ChordQuality::Dominant => "7",
            ChordQuality::Diminished => {
                if self.extensions.contains(&7) { "dim7" } else { "dim" }
            }
            ChordQuality::HalfDiminished => "m7b5",
            ChordQuality::Augmented => "aug",
            ChordQuality::Suspended => "sus",
        };
        format!("{}{}", root_name, quality_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_tension_ordering() {
        // dim7 > dom7 > maj7
        let dim = Chord::from_name("Cdim").unwrap();
        let dom = Chord::from_name("G7").unwrap();
        let maj = Chord::from_name("Cmaj7").unwrap();
        assert!(dim.tension() > dom.tension());
        assert!(dom.tension() > maj.tension());
    }

    #[test]
    fn test_chord_consonance() {
        let c_maj = Chord::new(Pitch { midi: 60 }, ChordQuality::Major);
        assert!(c_maj.consonance() > 0.5);
    }

    #[test]
    fn test_chord_from_name() {
        let dm7 = Chord::from_name("Dm7").unwrap();
        assert_eq!(dm7.root.midi, 62); // D4
        assert_eq!(dm7.quality, ChordQuality::Minor);

        let g7 = Chord::from_name("G7").unwrap();
        assert_eq!(g7.root.midi, 67); // G4
        assert_eq!(g7.quality, ChordQuality::Dominant);

        let cmaj7 = Chord::from_name("Cmaj7").unwrap();
        assert_eq!(cmaj7.root.midi, 60); // C4
        assert_eq!(cmaj7.quality, ChordQuality::Major);
    }

    #[test]
    fn test_chord_pitches() {
        let c_maj = Chord::new(Pitch { midi: 60 }, ChordQuality::Major);
        let pitches = c_maj.pitches();
        assert_eq!(pitches.len(), 3); // C, E, G
        assert_eq!(pitches[0].midi, 60);
        assert_eq!(pitches[1].midi, 64);
        assert_eq!(pitches[2].midi, 67);
    }

    #[test]
    fn test_voice_leading_cost() {
        // C to Cm: should be low (just E -> Eb = 1 semitone)
        let c_maj = Chord::from_name("C").unwrap();
        let c_min = Chord::from_name("Cm").unwrap();
        let cost = c_maj.voice_leading_cost(&c_min);
        assert!(cost < 2.0, "C -> Cm cost should be low, got {}", cost);
    }

    #[test]
    fn test_dominant_pitches() {
        let g7 = Chord::from_name("G7").unwrap();
        let pitches = g7.pitches();
        assert_eq!(pitches.len(), 4); // G, B, D, F
    }
}
