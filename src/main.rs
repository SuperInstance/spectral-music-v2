use spectral_music_v2::*;

fn main() {
    println!("🎵 spectral-music-v2: Complete Spectral Music Theory Engine\n");

    // Module 1: Pitch
    let a4 = pitch::Pitch::from_note('A', 4, false);
    println!("A4 = {} Hz (MIDI {})", a4.frequency(), a4.midi);

    let p5 = pitch::Interval { semitones: 7 };
    println!("P5 consonance: {:.3}", p5.consonance());
    println!("P5 voice-leading distance: {:.1}", p5.voice_leading_distance());

    // Module 2: Chord tension
    let chords = ["Cmaj7", "Dm7", "G7", "Cdim", "Caug"];
    println!("\n📊 Chord Tensions:");
    for name in &chords {
        let c = chord::Chord::from_name(name).unwrap();
        println!("  {} → tension={:.3} consonance={:.3}", name, c.tension(), c.consonance());
    }

    // Module 3: Progressions
    println!("\n🎼 Progression Analysis:");
    let progs = [
        ("ii-V-I", progression::two_five_one()),
        ("Circle of Fifths", progression::circle_of_fifths()),
        ("I-vi-ii-V", progression::one_six_two_five()),
        ("I-IV-V-I", progression::one_four_five_one()),
    ];

    for (name, prog) in &progs {
        let cr = prog.conservation();
        let rq = prog.resolution_quality();
        let smooth = prog.avg_smoothness();
        println!("  {} → CR={:.3} resolution={:.3} smoothness={:.3}", name, cr, rq, smooth);
    }

    // Module 4: Spline voice leading
    println!("\n🎹 Spline Voice Leading (ii-V-I, 4 voices):");
    let chords = vec![
        chord::Chord::from_name("Dm7").unwrap(),
        chord::Chord::from_name("G7").unwrap(),
        chord::Chord::from_name("Cmaj7").unwrap(),
    ];
    let svl = spline_voice_leading::SplineVoiceLeading::smoothest(chords, 4);
    println!("  Total cost: {:.1}", svl.total_cost());
    println!("  Conservation: {:.3}", svl.conservation());
    let curv = svl.curvature();
    for (i, c) in curv.iter().enumerate() {
        println!("  Voice {} curvature: {:.3}", i, c);
    }

    // Module 5: Fibonacci Harmony
    println!("\n🌀 Fibonacci Harmony:");
    println!("  φ = {:.6}", fibonacci_harmony::FibonacciHarmony::phi());
    println!("  1/φ = {:.6}", 1.0 / fibonacci_harmony::FibonacciHarmony::phi());

    println!("\n🏆 Jazz Progressions ranked by Conservation Ratio:");
    let ranked = fibonacci_harmony::FibonacciHarmony::jazz_progressions_by_cr();
    for (i, (name, cr)) in ranked.iter().enumerate() {
        let bar = "█".repeat((cr * 30.0) as usize);
        println!("  {:2}. {:20} CR = {:.3} {}", i + 1, name, cr, bar);
    }

    println!("\n✨ Music IS spectral graph theory.");
}
