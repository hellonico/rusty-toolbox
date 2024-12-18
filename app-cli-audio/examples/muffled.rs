use std::thread::sleep;
use std::time::Duration;
use kira::{
    manager::{
        AudioManager, AudioManagerSettings,
        backend::DefaultBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    track::TrackBuilder,
    effect::filter::FilterBuilder,
};
fn main() {

    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    // Create a mixer sub-track with a filter.
    let track = manager.add_sub_track({
        let mut builder = TrackBuilder::new();
        builder.add_effect(FilterBuilder::new().cutoff(1000.0));
        builder
    }).unwrap();
    // Play the sound on the track.
    let sound_data = StaticSoundData::from_file("app-cli-audio/astral-creepy-dark-logo-254198.mp3").unwrap().output_destination(&track);
    manager.play(sound_data).unwrap();
    sleep(Duration::from_secs(10));
}