use std::path::PathBuf;
use app_cli_overlay_text::{blend_text_to_image, blend_text_to_image_};
use clap::{Arg, Command};

fn main() {
    // Define CLI arguments using `clap`
    let matches = Command::new("Image Overlay Text")
        .version("1.0")
        .author("Nicolas Modrzyk <hellonico@gmail.com>")
        .about("Adds handwritten-like text overlay to an image")
        .arg(Arg::new("image")
            .short('i')
            .long("image")
            .value_name("IMAGE_PATH")
            .help("Path to the input image")
            .required(true))
        .arg(Arg::new("font")
            .short('f')
            .long("font")
            .value_name("FONT_PATH")
            .help("Path to the font file")
            .required(true))
        .arg(Arg::new("text")
            .short('t')
            .long("text")
            .value_name("TEXT")
            .help("Text to overlay on the image")
            .required(true))
        .arg(Arg::new("x")
            .short('x')
            .long("x-coord")
            .value_name("X")
            .help("X-coordinate for text placement")
            .default_value("50.0"))
        .arg(Arg::new("y")
            .short('y')
            .long("y-coord")
            .value_name("Y")
            .help("Y-coordinate for text placement")
            .default_value("35.0"))
        .arg(Arg::new("scale")
            .short('s')
            .long("scale")
            .value_name("SCALE_FACTOR")
            .help("Scale factor for the font size (relative to image width)")
            .default_value("0.1"))
        .arg(Arg::new("thickness")
            .short('k')
            .long("thickness")
            .value_name("THICKNESS_FACTOR")
            .help("Thickness for the font")
            .default_value("1"))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("OUTPUT_PATH")
            .help("Path to save the output image")
            .default_value("output.png"))
        .arg(Arg::new("color")
            .short('c')
            .long("color")
            .value_name("COLOR")
            .help("Hex color code for font")
            .default_value("#FF4500"))
        .get_matches();

    // Parse arguments
    let image_path = matches.get_one::<String>("image").unwrap();
    let font_path = matches.get_one::<String>("font").unwrap();
    let text = matches.get_one::<String>("text").unwrap();
    let x: f32 = matches.get_one::<String>("x").unwrap().parse().expect("Invalid X-coordinate");
    let y: f32 = matches.get_one::<String>("y").unwrap().parse().expect("Invalid Y-coordinate");
    let scale_factor: f32 = matches.get_one::<String>("scale").unwrap().parse().expect("Invalid scale factor");
    let output_path =  matches.get_one::<String>("output").unwrap();
    let thickness:u32 =  matches.get_one::<String>("thickness").unwrap().parse().expect("Invalid thickness");
    let color:&str =  matches.get_one::<String>("color").expect("Invalid color");

    let output = blend_text_to_image_(image_path, &PathBuf::from(font_path), text, x, y, scale_factor, thickness, color );
    output.save(output_path).expect("Failed to save image");
}
