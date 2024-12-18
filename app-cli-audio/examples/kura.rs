use std::thread::sleep;

fn main() {
    use std::time::Duration;

    use kira::{
        manager::{
            AudioManager, AudioManagerSettings,
            backend::DefaultBackend,
        },
        sound::static_sound::{StaticSoundData, StaticSoundSettings},
        tween::Tween,
    };

    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let sound_data = StaticSoundData::from_file("app-cli-audio/astral-creepy-dark-logo-254198.mp3").unwrap();
    let mut sound = manager.play(sound_data).unwrap();
    // Start smoothly adjusting the playback rate parameter.
    sound.set_playback_rate(
        2.0,
        Tween {
            duration: Duration::from_secs(3),
            ..Default::default()
        },
    );
    sleep(Duration::from_secs(10));

}
