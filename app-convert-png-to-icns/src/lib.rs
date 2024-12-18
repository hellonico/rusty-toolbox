use icns::{IconFamily, IconType, Image};
use image::io::Reader as ImageReader;
use lib_ffmpeg_utils::append_to_home_log;
use std::fs::File;
use std::io::{BufReader, Write};
use tempfile::NamedTempFile;

pub fn png_to_icns(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load the PNG image
    let img = ImageReader::open(input_path)?.decode()?;

    // Create an IconFamily for the ICNS file
    let mut icon_family = IconFamily::new();
    // let mut icon_family = IconFamily::read(file).unwrap();

    // Define the sizes needed for ICNS
    let sizes = [
        (16, IconType::RGB24_16x16),
        (32, IconType::RGB24_32x32),
        (64, IconType::RGB24_48x48),
        (128, IconType::RGBA32_128x128),
    ];

    // Resize the image and add to the IconFamily
    for (size, icon_type) in sizes {
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        for (size, icon_type) in sizes {
            let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
            let mut temp = NamedTempFile::with_suffix_in(".png", std::env::temp_dir()).unwrap();
            resized.save(temp.path().display().to_string())?;
            let file = BufReader::new(File::open(temp).unwrap());
            let image = Image::read_png(file).unwrap();
            icon_family.add_icon(&image).unwrap()
        }
    }

    // Save the ICNS file
    let mut file = File::create(output_path)?;
    icon_family.write(file)?;

    append_to_home_log("ICNS file created successfully!".parse().unwrap());
    Ok(())
}
