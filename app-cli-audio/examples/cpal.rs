use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    // Generate a sine wave
    let sample_rate = config.sample_rate().0 as f32;
    let mut t = 0f32;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                let amplitude = 0.2;
                *sample = amplitude * (t * 2.0 * std::f32::consts::PI * 440.0 / sample_rate).sin();
                t += 1.0;
            }
        },
        |err| eprintln!("Error occurred: {}", err),
        None,
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(5));
}
