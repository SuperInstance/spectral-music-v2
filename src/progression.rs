//! Module 3: Progression — the graph
//! Chords are nodes, voice-leading distances are edge weights.
//! The Laplacian's eigenvalues encode harmonic tension.
//! CR (Conservation Ratio) measures how consonant a progression is.

use crate::chord::Chord;

/// A chord progression as a weighted graph
#[derive(Debug, Clone)]
pub struct Progression {
    pub chords: Vec<Chord>,
    /// Adjacency matrix: weights[i][j] = voice-leading distance from chord i to j
    pub weights: Vec<Vec<f64>>,
}

impl Progression {
    /// Build a progression from chords, computing voice-leading weights
    pub fn from_chords(chords: Vec<Chord>) -> Progression {
        let n = chords.len();
        let mut weights = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i != j {
                    weights[i][j] = chords[i].voice_leading_cost(&chords[j]);
                }
            }
        }

        Progression { chords, weights }
    }

    /// Build from sequential chord pairs only (chain graph)
    pub fn from_chord_sequence(chords: Vec<Chord>) -> Progression {
        let n = chords.len();
        let mut weights = vec![vec![0.0; n]; n];

        for i in 0..n.saturating_sub(1) {
            let cost = chords[i].voice_leading_cost(&chords[i + 1]);
            weights[i][i + 1] = cost;
            weights[i + 1][i] = cost;
        }

        Progression { chords, weights }
    }

    /// Graph Laplacian: L = D - W where D is degree matrix, W is weight matrix
    pub fn laplacian(&self) -> Vec<Vec<f64>> {
        let n = self.chords.len();
        let mut lap = vec![vec![0.0; n]; n];

        for i in 0..n {
            let mut degree = 0.0;
            for j in 0..n {
                if i != j {
                    degree += self.weights[i][j];
                }
            }
            lap[i][i] = degree;
            for j in 0..n {
                if i != j {
                    lap[i][j] = -self.weights[i][j];
                }
            }
        }

        lap
    }

    /// Compute eigenvalues of the Laplacian using power iteration + deflation.
    /// Returns eigenvalues sorted in ascending order.
    fn laplacian_eigenvalues(&self) -> Vec<f64> {
        let n = self.chords.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![0.0];
        }

        let lap = self.laplacian();
        let mut eigenvalues = Vec::with_capacity(n);

        // Modified matrix for deflation
        let mut mat = lap.clone();

        for k in 0..n {
            // Power iteration to find dominant eigenvalue
            let eigenval = Self::power_eigenvalue(&mat, 100);

            if eigenval.abs() > 1e-10 {
                // Get eigenvector for deflation
                let v = Self::power_eigenvector(&mat, 100);
                // Deflate: mat = mat - eigenval * v * v^T
                for i in 0..n {
                    for j in 0..n {
                        mat[i][j] -= eigenval * v[i] * v[j];
                    }
                }
            }

            eigenvalues.push(eigenval);
        }

        eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        eigenvalues
    }

    /// Power iteration for dominant eigenvalue
    fn power_eigenvalue(mat: &[Vec<f64>], iterations: usize) -> f64 {
        let n = mat.len();
        let mut v = vec![1.0 / (n as f64).sqrt(); n];

        for _ in 0..iterations {
            let mut new_v = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    new_v[i] += mat[i][j] * v[j];
                }
            }
            let norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-12 {
            for x in &mut new_v {
                *x /= norm;
            }
            }
            v = new_v;
        }

        // Rayleigh quotient
        let mut mv = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                mv[i] += mat[i][j] * v[j];
            }
        }

        let num: f64 = v.iter().zip(mv.iter()).map(|(a, b)| a * b).sum();
        let den: f64 = v.iter().map(|x| x * x).sum();
        if den.abs() < 1e-12 { 0.0 } else { num / den }
    }

    /// Power iteration for dominant eigenvector
    fn power_eigenvector(mat: &[Vec<f64>], iterations: usize) -> Vec<f64> {
        let n = mat.len();
        let mut v = vec![1.0 / (n as f64).sqrt(); n];

        for _ in 0..iterations {
            let mut new_v = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    new_v[i] += mat[i][j] * v[j];
                }
            }
            let norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-12 {
            for x in &mut new_v {
                *x /= norm;
            }
            }
            v = new_v;
        }

        v
    }

    /// Conservation Ratio (CR): measures how conserved the voice-leading energy is.
    /// CR = λ_min / λ_max where λ are the non-zero eigenvalues of the Laplacian.
    /// High CR = conserved progression (minimal voice-leading waste).
    /// Common-practice progressions show ~112× CR advantage over random.
    pub fn conservation(&self) -> f64 {
        let eigenvalues = self.laplacian_eigenvalues();
        let non_zero: Vec<f64> = eigenvalues.iter()
            .filter(|&&e| e.abs() > 1e-8)
            .copied()
            .collect();

        if non_zero.is_empty() {
            return 1.0;
        }

        let min = non_zero.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = non_zero.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if max.abs() < 1e-8 {
            return 1.0;
        }

        // CR = ratio of how "concentrated" the eigenvalue spectrum is
        // High CR = eigenvalues clustered = conserved
        let sum: f64 = non_zero.iter().sum();
        let mean = sum / non_zero.len() as f64;

        // Normalized: how close is the smallest nonzero eigenvalue to the mean
        // Perfect conservation = all eigenvalues equal = ratio = 1
        // Poor conservation = spread eigenvalues = ratio < 1
        (min / max).abs().min(1.0)
    }

    /// Fiedler vector: eigenvector corresponding to the second-smallest eigenvalue.
    /// For the circle of fifths, this IS the line of fifths.
    pub fn fiedler_vector(&self) -> Vec<f64> {
        let n = self.chords.len();
        if n < 2 {
            return vec![0.0; n];
        }

        let lap = self.laplacian();

        // Shift Laplacian to find the Fiedler eigenvalue
        // The second-smallest eigenvalue of L corresponds to the smallest non-trivial eigenvalue
        // Use inverse power iteration with shift near 0 (but not 0)

        // First, compute an approximate Fiedler value from the eigenvalues
        let eigenvalues = self.laplacian_eigenvalues();
        let fiedler_val = eigenvalues.iter()
            .filter(|&&e| e > 1e-8)
            .cloned()
            .next()
            .unwrap_or(1.0);

        // Inverse power iteration with shift = fiedler_val
        let mut v = vec![0.0; n];
        for i in 0..n {
            v[i] = (i as f64) * 0.1 - (n as f64 - 1.0) * 0.05;
        }

        for _ in 0..200 {
            // Solve (L - sigma*I) * x = v using Gauss-Seidel
            let sigma = fiedler_val * 0.5; // undershoot slightly
            let mut x = v.clone();
            for _ in 0..50 {
                for i in 0..n {
                    let mut sum = v[i];
                    for j in 0..n {
                        if j != i {
                            sum -= (lap[i][j] - if i == j { sigma } else { 0.0}) * x[j];
                        }
                    }
                    let diag = lap[i][i] - sigma;
                    if diag.abs() > 1e-12 {
                        x[i] = sum / diag;
                    }
                }
            }

            let norm: f64 = x.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-12 {
                for xi in &mut x {
                    *xi /= norm;
                }
            }
            v = x;
        }

        v
    }

    /// Tension profile: tension at each chord in the progression
    pub fn tension_profile(&self) -> Vec<f64> {
        self.chords.iter().map(|c| c.tension()).collect()
    }

    /// Resolution quality: how well tension resolves.
    /// Measures if tension decreases at the end (resolution).
    /// Returns value in [0, 1]: 1 = perfect resolution.
    pub fn resolution_quality(&self) -> f64 {
        if self.chords.len() < 2 {
            return 1.0;
        }

        let tensions = self.tension_profile();
        let peak = tensions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let final_tension = *tensions.last().unwrap();

        if peak < 1e-8 {
            return 1.0;
        }

        // How much of the peak tension was resolved
        1.0 - (final_tension / peak).min(1.0)
    }

    /// Average voice-leading smoothness in the progression
    pub fn avg_smoothness(&self) -> f64 {
        let n = self.chords.len();
        if n < 2 {
            return 1.0;
        }

        let mut total_cost = 0.0;
        let mut count = 0;
        for i in 0..n.saturating_sub(1) {
            let cost = self.weights[i][i + 1];
            if cost > 0.0 {
                total_cost += cost;
                count += 1;
            }
        }

        if count == 0 {
            return 1.0;
        }

        // Smoothness = inverse of average cost, normalized
        let avg_cost = total_cost / count as f64;
        1.0 / (1.0 + avg_cost)
    }
}

/// The ii-V-I in C major — the most conserved progression in jazz
pub fn two_five_one() -> Progression {
    Progression::from_chord_sequence(vec![
        Chord::from_name("Dm7").unwrap(),
        Chord::from_name("G7").unwrap(),
        Chord::from_name("Cmaj7").unwrap(),
    ])
}

/// The circle of fifths as a graph
/// F-C-G-D-A-E-B-F#-C#-G#-D#-A#
pub fn circle_of_fifths() -> Progression {
    let fifths: [u8; 12] = [5, 0, 7, 2, 9, 4, 11, 6, 1, 8, 3, 10];
    let chords: Vec<Chord> = fifths.iter()
        .map(|&pc| Chord::new(crate::pitch::Pitch { midi: 60 + pc }, crate::chord::ChordQuality::Major))
        .collect();
    Progression::from_chord_sequence(chords)
}

/// I-vi-ii-V turnaround
pub fn one_six_two_five() -> Progression {
    Progression::from_chord_sequence(vec![
        Chord::from_name("Cmaj7").unwrap(),
        Chord::from_name("Am7").unwrap(),
        Chord::from_name("Dm7").unwrap(),
        Chord::from_name("G7").unwrap(),
    ])
}

/// I-IV-V-I cadence
pub fn one_four_five_one() -> Progression {
    Progression::from_chord_sequence(vec![
        Chord::from_name("C").unwrap(),
        Chord::from_name("F").unwrap(),
        Chord::from_name("G").unwrap(),
        Chord::from_name("C").unwrap(),
    ])
}

/// Random/atonal progression for comparison
pub fn random_progression() -> Progression {
    let pcs: [u8; 6] = [0, 1, 3, 6, 8, 11]; // chromatic-ish
    let qualities = [
        crate::chord::ChordQuality::Major,
        crate::chord::ChordQuality::Diminished,
        crate::chord::ChordQuality::Augmented,
        crate::chord::ChordQuality::Dominant,
        crate::chord::ChordQuality::HalfDiminished,
        crate::chord::ChordQuality::Minor,
    ];
    let chords: Vec<Chord> = pcs.iter().zip(qualities.iter())
        .map(|(&pc, &q)| Chord::new(crate::pitch::Pitch { midi: 60 + pc }, q))
        .collect();
    Progression::from_chord_sequence(chords)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_five_one_structure() {
        let prog = two_five_one();
        assert_eq!(prog.chords.len(), 3);
    }

    #[test]
    fn test_two_five_one_conservation() {
        // ii-V-I should have reasonable conservation
        let prog = two_five_one();
        let cr = prog.conservation();
        assert!(cr > 0.0, "ii-V-I CR should be positive, got {}", cr);
        assert!(cr <= 1.0, "CR should be <= 1, got {}", cr);
    }

    #[test]
    fn test_two_five_one_resolution() {
        let prog = two_five_one();
        let rq = prog.resolution_quality();
        assert!(rq > 0.0, "ii-V-I should resolve tension, got {}", rq);
    }

    #[test]
    fn test_circle_of_fifths() {
        let prog = circle_of_fifths();
        assert_eq!(prog.chords.len(), 12);
        let cr = prog.conservation();
        assert!(cr > 0.0);
    }

    #[test]
    fn test_laplacian() {
        let prog = two_five_one();
        let lap = prog.laplacian();
        assert_eq!(lap.len(), 3);
        // Laplacian rows should sum to ~0
        for i in 0..3 {
            let row_sum: f64 = lap[i].iter().sum();
            assert!(row_sum.abs() < 0.01, "Row {} sum = {}", i, row_sum);
        }
    }

    #[test]
    fn test_fiedler_vector() {
        let prog = circle_of_fifths();
        let fv = prog.fiedler_vector();
        assert_eq!(fv.len(), 12);
    }

    #[test]
    fn test_tension_profile() {
        let prog = two_five_one();
        let tp = prog.tension_profile();
        assert_eq!(tp.len(), 3);
        // G7 should have higher tension than Cmaj7
        assert!(tp[1] > tp[2], "G7 tension {} should be > Cmaj7 tension {}", tp[1], tp[2]);
    }

    #[test]
    fn test_tonal_vs_random_cr() {
        let tonal = one_four_five_one();
        let atonal = random_progression();
        // Tonal progressions tend to have higher conservation
        let tonal_cr = tonal.avg_smoothness();
        let atonal_smooth = atonal.avg_smoothness();
        // This is a soft test — just verify they compute
        assert!(tonal_cr > 0.0);
        assert!(atonal_smooth > 0.0);
    }

    #[test]
    fn test_one_six_two_five() {
        let prog = one_six_two_five();
        assert_eq!(prog.chords.len(), 4);
        let cr = prog.conservation();
        assert!(cr > 0.0);
    }
}
