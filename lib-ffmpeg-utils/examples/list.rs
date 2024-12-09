use lib_ffmpeg_utils::devices::DeviceLister;

fn main() {
    // Create a new DeviceLister instance
    let device_lister = DeviceLister::new();

    // Get and print video devices
    let video_devices = device_lister.get_video_devices();
    println!("Video Devices: {:?}", video_devices);

    // Get and print audio devices
    let audio_devices = device_lister.get_audio_devices();
    println!("Audio Devices: {:?}", audio_devices);
}
