mod muffled;

use rodio::{source::SineWave, OutputStream, Sink, Source};
use std::time::Duration;

fn main() {
    // Initialize audio stream
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Create sinks for each track
    let track1 = Sink::try_new(&stream_handle).unwrap();
    let track2 = Sink::try_new(&stream_handle).unwrap();

    // Generate audio for the first track
    track1.append(SineWave::new(440.0).take_duration(Duration::from_secs(5)));

    // Generate audio for the second track
    track2.append(SineWave::new(220.0).take_duration(Duration::from_secs(5)));

    // Play tracks
    track1.detach();
    track2.detach();

    std::thread::sleep(Duration::from_secs(6));
}
