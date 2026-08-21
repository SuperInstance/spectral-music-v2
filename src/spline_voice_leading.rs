//! Module 4: SplineVoiceLeading — voice leading as splines through pitch-space
//! Each voice is a spline, control points = chord tones.
//! The smoothness of the spline = quality of voice leading.

use crate::chord::Chord;
use crate::pitch::Pitch;

/// A voice event: a note played by one voice
#[derive(Debug, Clone)]
pub struct VoiceEvent {
    pub time: f64,
    pub voice: usize,
    pub pitch: Pitch,
    pub duration: f64,
}

/// Voice leading as spline through pitch-space
#[derive(Debug, Clone)]
pub struct SplineVoiceLeading {
    pub voices: usize,
    pub chords: Vec<Chord>,
    /// One path per voice through all chords
    pub paths: Vec<Vec<Pitch>>,
}

impl SplineVoiceLeading {
    /// Find the smoothest voice leading (minimize total movement).
    /// Uses a greedy nearest-neighbor assignment per voice.
    pub fn smoothest(chords: Vec<Chord>, voices: usize) -> SplineVoiceLeading {
        if chords.is_empty() {
            return SplineVoiceLeading {
                voices,
                chords,
                paths: vec![vec![]; voices],
            };
        }

        let n = chords.len();
        let mut paths: Vec<Vec<Pitch>> = vec![vec![]; voices];

        // Initialize: assign voices to first chord
        let first_pitches = Self::assign_voices(&chords[0], voices);
        for v in 0..voices {
            paths[v].push(first_pitches[v]);
        }

        // For each subsequent chord, find minimal-movement assignment
        for i in 1..n {
            let current_positions: Vec<Pitch> = paths.iter().map(|p| *p.last().unwrap()).collect();
            let assignments = Self::minimal_assignment(&current_positions, &chords[i], voices);
            for v in 0..voices {
                paths[v].push(assignments[v]);
            }
        }

        SplineVoiceLeading {
            voices,
            chords,
            paths,
        }
    }

    /// Assign voices to chord pitches, duplicating root if needed
    fn assign_voices(chord: &Chord, voices: usize) -> Vec<Pitch> {
        let pitches = chord.pitches();
        let mut result = Vec::with_capacity(voices);
        for i in 0..voices {
            result.push(pitches[i % pitches.len()]);
        }
        result
    }

    /// Find minimal-movement assignment from current positions to chord pitches
    fn minimal_assignment(current: &[Pitch], target_chord: &Chord, voices: usize) -> Vec<Pitch> {
        let target_pitches = target_chord.pitches();
        let n = voices;

        // Greedy: for each voice, pick the closest target pitch
        let mut used = vec![false; target_pitches.len()];
        let mut result = vec![current[0]; n];

        // Try all permutations for small n, greedy for larger
        if n <= 4 {
            // Brute force for small voice counts
            let mut best_assignment = vec![0; n];
            let mut best_cost = f64::INFINITY;

            Self::enumerate_assignments(n, target_pitches.len(), &mut vec![0; n], 0, &mut best_assignment, &mut best_cost, current, &target_pitches);

            for i in 0..n {
                result[i] = target_pitches[best_assignment[i] % target_pitches.len()];
            }
        } else {
            // Greedy nearest neighbor, enforcing a distinct target pitch per
            // voice while any unclaimed pitch remains.
            //
            // When there are more voices than distinct target pitches, doubling
            // is unavoidable; once every target pitch has been claimed we relax
            // the constraint and allow reuse, again choosing the closest pitch.
            let mut used_count = 0;
            for i in 0..n {
                let mut best_j = 0;
                let mut best_dist = f64::INFINITY;
                let allow_reuse = used_count >= target_pitches.len();
                for j in 0..target_pitches.len() {
                    if !allow_reuse && used[j] {
                        continue;
                    }
                    let interval = current[i].interval_to(&target_pitches[j]);
                    let dist = interval.minimal_distance();
                    if dist < best_dist {
                        best_dist = dist;
                        best_j = j;
                    }
                }
                result[i] = target_pitches[best_j];
                if !used[best_j] {
                    used[best_j] = true;
                    used_count += 1;
                }
            }
        }

        result
    }

    /// Enumerate all assignments recursively
    fn enumerate_assignments(
        n: usize,
        target_len: usize,
        current: &mut Vec<usize>,
        pos: usize,
        best_assignment: &mut Vec<usize>,
        best_cost: &mut f64,
        positions: &[Pitch],
        targets: &[Pitch],
    ) {
        if pos == n {
            let cost: f64 = (0..n)
                .map(|i| {
                    let interval = positions[i].interval_to(&targets[current[i] % target_len]);
                    interval.minimal_distance()
                })
                .sum();
            if cost < *best_cost {
                *best_cost = cost;
                *best_assignment = current.clone();
            }
            return;
        }

        for j in 0..target_len {
            current[pos] = j;
            Self::enumerate_assignments(n, target_len, current, pos + 1, best_assignment, best_cost, positions, targets);
        }
    }

    /// Compute the spline curvature of each voice.
    /// Uses second differences: κ(t) ≈ |y(t+h) - 2y(t) + y(t-h)| / h²
    pub fn curvature(&self) -> Vec<f64> {
        let mut curvatures = Vec::with_capacity(self.voices);

        for v in 0..self.voices {
            let path = &self.paths[v];
            if path.len() < 3 {
                curvatures.push(0.0);
                continue;
            }

            let mut total_curvature = 0.0;
            for i in 1..path.len().saturating_sub(1) {
                let second_diff = (path[i + 1].midi as f64
                    - 2.0 * path[i].midi as f64
                    + path[i - 1].midi as f64)
                    .abs();
                total_curvature += second_diff;
            }

            let avg_curvature = total_curvature / (path.len() - 2).max(1) as f64;
            curvatures.push(avg_curvature);
        }

        curvatures
    }

    /// Conservation ratio of the voice-leading paths.
    /// How smooth are the paths overall?
    pub fn conservation(&self) -> f64 {
        let mut total_movement = 0.0;
        let mut total_curvature = 0.0;

        for v in 0..self.voices {
            let path = &self.paths[v];
            for i in 1..path.len() {
                let interval = path[i - 1].interval_to(&path[i]);
                total_movement += interval.minimal_distance();
            }
        }

        let curvatures = self.curvature();
        for &c in &curvatures {
            total_curvature += c;
        }

        // Conservation = low curvature / movement ratio
        if total_movement < 1e-8 {
            return 1.0;
        }

        let smoothness = 1.0 / (1.0 + total_curvature);
        smoothness
    }

    /// Generate MIDI-like voice events
    pub fn to_events(&self) -> Vec<VoiceEvent> {
        let mut events = Vec::new();
        let beat_duration = 1.0;

        for (chord_idx, _chord) in self.chords.iter().enumerate() {
            let time = chord_idx as f64 * beat_duration;
            for v in 0..self.voices {
                if chord_idx < self.paths[v].len() {
                    events.push(VoiceEvent {
                        time,
                        voice: v,
                        pitch: self.paths[v][chord_idx],
                        duration: beat_duration,
                    });
                }
            }
        }

        events
    }

    /// Total voice leading cost
    pub fn total_cost(&self) -> f64 {
        let mut total = 0.0;
        for v in 0..self.voices {
            let path = &self.paths[v];
            for i in 1..path.len() {
                let interval = path[i - 1].interval_to(&path[i]);
                total += interval.minimal_distance();
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_smoothest_voice_leading() {
        let chords = vec![
            Chord::from_name("C").unwrap(),
            Chord::from_name("F").unwrap(),
            Chord::from_name("G").unwrap(),
            Chord::from_name("C").unwrap(),
        ];
        let svl = SplineVoiceLeading::smoothest(chords, 3);
        assert_eq!(svl.paths.len(), 3);
        assert_eq!(svl.paths[0].len(), 4);
    }

    #[test]
    fn test_curvature() {
        let chords = vec![
            Chord::from_name("C").unwrap(),
            Chord::from_name("C").unwrap(),
            Chord::from_name("C").unwrap(),
        ];
        let svl = SplineVoiceLeading::smoothest(chords, 3);
        let curv = svl.curvature();
        // Repeated chords should have zero curvature
        for &c in &curv {
            assert!(c < 0.01, "Repeated chord curvature should be ~0, got {}", c);
        }
    }

    #[test]
    fn test_conservation() {
        let chords = vec![
            Chord::from_name("C").unwrap(),
            Chord::from_name("C").unwrap(),
        ];
        let svl = SplineVoiceLeading::smoothest(chords, 3);
        // Same chord = perfect conservation
        assert!(svl.conservation() > 0.9);
    }

    #[test]
    fn test_to_events() {
        let chords = vec![
            Chord::from_name("C").unwrap(),
            Chord::from_name("G").unwrap(),
        ];
        let svl = SplineVoiceLeading::smoothest(chords, 3);
        let events = svl.to_events();
        assert_eq!(events.len(), 6); // 2 chords × 3 voices
    }

    #[test]
    fn test_smooth_leading_minimal() {
        // ii-V-I should have very smooth voice leading
        let chords = vec![
            Chord::from_name("Dm7").unwrap(),
            Chord::from_name("G7").unwrap(),
            Chord::from_name("Cmaj7").unwrap(),
        ];
        let svl = SplineVoiceLeading::smoothest(chords, 4);
        let cost = svl.total_cost();
        // ii-V-I in close position: total should be quite low
        assert!(cost < 12.0, "ii-V-I total cost should be low, got {}", cost);
    }

    #[test]
    fn test_minimal_assignment_brute_force_optimal() {
        // Hand-computable optimal assignment for the brute-force path (n <= 4).
        // Current: C4 E4 G4. Target: G major (G4 B4 D5).
        // Optimal permutation: C4 -> B4 (1), E4 -> D5 (2), G4 -> G4 (0), total 3.
        let current = vec![
            Pitch { midi: 60 }, // C4
            Pitch { midi: 64 }, // E4
            Pitch { midi: 67 }, // G4
        ];
        let target = Chord::from_name("G").unwrap();
        let assignment = SplineVoiceLeading::minimal_assignment(&current, &target, 3);
        assert_eq!(
            assignment,
            vec![
                Pitch { midi: 71 }, // B4
                Pitch { midi: 74 }, // D5
                Pitch { midi: 67 }, // G4
            ]
        );
    }

    #[test]
    fn test_minimal_assignment_greedy_distinct_targets() {
        // Greedy path (n > 4) must not assign two voices to the same target pitch
        // while an unclaimed target pitch still exists.
        // Five voices moving to a C major triad (only 3 distinct pitches).
        //
        // Voice 0 claims C4. Voices 1 and 2 are placed so that their closest
        // target would also be C4; the fixed implementation must skip the
        // already-claimed C4 and pick distinct, still-free targets instead.
        let current = vec![
            Pitch { midi: 60 }, // C4 -> C4
            Pitch { midi: 59 }, // B3 -> G4 (C4 is already claimed)
            Pitch { midi: 62 }, // D4 -> E4 (C4 is already claimed)
            Pitch { midi: 48 }, // C3 -> C4 once all pitches are claimed
            Pitch { midi: 52 }, // E3 -> E4 once all pitches are claimed
        ];
        let target = Chord::from_name("C").unwrap();
        let assignment = SplineVoiceLeading::minimal_assignment(&current, &target, 5);
        assert_eq!(assignment.len(), 5);

        // The first three voices should claim all three distinct target pitches.
        let first_three: std::collections::HashSet<_> =
            assignment[..3].iter().map(|p| p.midi).collect();
        assert_eq!(
            first_three.len(),
            3,
            "first three voices should each get a distinct target pitch"
        );

        // Voice 1 must not collide with voice 0 while G4 is still free.
        assert_ne!(
            assignment[1].midi, 60,
            "greedy should avoid reusing C4 while other targets are free"
        );
        // Voice 2 must not collide with voice 0 while E4 is still free.
        assert_ne!(
            assignment[2].midi, 60,
            "greedy should avoid reusing C4 while other targets are free"
        );
    }
}
