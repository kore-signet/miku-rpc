// note: requires a patched OC2 built with PRs #137 & #139; + the soundCardCoolDown must be set to 0

use miku_rpc::wrappers::{SoundCard, SoundInterface};
use miku_rpc::DeviceBus;
use std::thread;
use std::time::{Duration, Instant};

const SCALE: [f64; 25] = [
    0.5, 0.529732, 0.561231, 0.594604, 0.629961, 0.667420, 0.707107, 0.749154, 0.793701, 0.840896,
    0.890899, 0.943874, 1.0, 1.059463, 1.122462, 1.189207, 1.259921, 1.334840, 1.414214, 1.498307,
    1.587401, 1.681793, 1.781797, 1.887749, 2.0,
];
const PITCHES: [&str; 25] = [
    "F#", "G", "G#", "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#2", "G2", "G#2", "A2",
    "A#2", "B2", "C2", "C#2", "D2", "D#2", "E2", "F2", "F#3",
];

struct Note {
    pitch: f64,
    idx: usize,
    duration: Duration,
}

macro_rules! note {
    ($pitch:expr, $len:tt ms) => {
        Note {
            pitch: SCALE[$pitch],
            idx: $pitch,
            duration: Duration::from_millis($len),
        }
    };
}

fn main() -> std::io::Result<()> {
    let mut bus = DeviceBus::new("/dev/hvc0")?;

    let card: SoundCard = bus.wrap()?.expect("a file import/export card is required!");

    let notes = vec![
        // C Eb F Gb G Bb C
        note!(6, 600 ms),
        note!(9, 300 ms),
        note!(11, 600 ms),
        note!(12, 300 ms),
        note!(13, 600 ms),
        note!(16, 300 ms),
        note!(18, 300 ms),
    ];

    for note in notes {
        let start = Instant::now();
        card.play_sound(&mut bus, "block.note_block.pling", 1.0, note.pitch)?;
        println!("x {}", PITCHES[note.idx]);

        if let Some(s) = note.duration.checked_sub(start.elapsed()) {
            thread::sleep(s);
        }
    }

    Ok(())
}
