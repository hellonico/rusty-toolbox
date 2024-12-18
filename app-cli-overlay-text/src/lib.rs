use image::{DynamicImage, ImageReader, Pixel, Rgba, RgbaImage};
use rusttype::{point, Font, Scale};
use std::fs;
use std::path::PathBuf;
use eframe::egui;
use eframe::egui::ColorImage;

pub fn load_image(image_path: &str) -> RgbaImage {
    println!("Loading image: {}", image_path);
    image::open(image_path)
        .expect("Failed to open image")
        .to_rgba8()
}
pub fn load_font(font_path: PathBuf) -> Font<'static> {
    println!("Loading font: {}", font_path.display());
    let font_data = fs::read(font_path).expect("Failed to read font file");
    let font = Font::try_from_vec(font_data).expect("Error loading font");
    font
}
pub fn blend_text_to_image_(
    image_path: &str,
    font_path: PathBuf,
    text: &str,
    x: f32,
    y: f32,
    scale_factor: f32,
    thickness: u32,  // New: Thickness for bold effect
    color_hex: &str, // Hex color for the text
) -> RgbaImage {

    let result = blend_text_to_image(
        load_image(image_path),
        load_font(font_path),
        text,
        x,
        y,
        scale_factor,
        thickness,
        color_hex,
    );
    result
}

pub fn blend_text_to_image(
    mut img: RgbaImage,
    font: Font,
    text: &str,
    x: f32,
    y: f32,
    scale_factor: f32,
    thickness: u32,  // New: Thickness for bold effect
    color_hex: &str, // Hex color for the text
) -> RgbaImage {
    let color = parse_hex_color(color_hex).expect("Invalid color hex string");

    // Scale for font size
    let scale = Scale {
        x: img.width() as f32 * scale_factor,
        y: img.width() as f32 * scale_factor,
    };

    // Get image dimensions
    let (width, height) = img.dimensions();

    // Split the text into lines for multi-line support
    let lines: Vec<&str> = text.split("\\n").collect();
    println!("{:?}", lines.to_vec());
    let line_height = scale.y + 5.0; // Adjust line spacing
    let mut current_y = y;

    // Render each line of text
    for line in lines {
        // Render with thickness
        for offset_x in -(thickness as i32)..=(thickness as i32) {
            for offset_y in -(thickness as i32)..=(thickness as i32) {
                // Avoid duplicating center render for performance
                if offset_x.abs() + offset_y.abs() <= thickness as i32 {
                    for glyph in font.layout(
                        line,
                        scale,
                        point(x + offset_x as f32, current_y + offset_y as f32),
                    ) {
                        if let Some(bounding_box) = glyph.pixel_bounding_box() {
                            glyph.draw(|gx, gy, v| {
                                let px = gx as i32 + bounding_box.min.x;
                                let py = gy as i32 + bounding_box.min.y;

                                if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                                    let alpha = (v * 255.0) as u8;

                                    // Get mutable reference to the pixel
                                    let pixel = img.get_pixel_mut(px as u32, py as u32);

                                    // Blend the pixel with text color
                                    let channels = color.channels();
                                    let color_with_alpha = [channels[0], channels[1], channels[2], alpha];
                                    blend(pixel, Rgba::from(color_with_alpha)); // Black with alpha blending
                                }
                            });
                        }
                    }
                }
            }
        }

        // Move Y position down for the next line
        current_y += line_height;
    }

    img
}

// Blend function
fn blend(base: &mut Rgba<u8>, overlay: Rgba<u8>) {
    let alpha = overlay[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    base[0] = (base[0] as f32 * inv_alpha + overlay[0] as f32 * alpha) as u8;
    base[1] = (base[1] as f32 * inv_alpha + overlay[1] as f32 * alpha) as u8;
    base[2] = (base[2] as f32 * inv_alpha + overlay[2] as f32 * alpha) as u8;
    base[3] = 255; // Set alpha to fully opaque
}

// Parse a hex color string into an Rgba<u8>
fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Rgba([r, g, b, 255])) // Fully opaque
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Rgba([r, g, b, a]))
        }
        _ => Some(Rgba([0, 0, 0, 0])),
    }
}


//
// pub fn load_image_as_bytes(image_path: &str) -> ColorImage {
//     // Load the image from the file path
//     let image = ImageReader::open(image_path)
//         .expect("Failed to open image")
//         .decode()
//         .expect("Failed to decode image");
//     load_image_as_bytes_(image.to_rgba8())
// }
//
// pub fn load_image_as_bytes_(image: RgbaImage) -> ColorImage {
//     println!("Loading image...{:?}", image.dimensions());
//     // let (width, height) = image.dimensions();
//     let pixels = image.as_flat_samples();
//     let (width, height) = image.dimensions();
//
//     // Convert dimensions from (u32, u32) to [usize; 2]
//     let dimensions = [width as usize, height as usize];
//     ColorImage::from_rgba_unmultiplied(
//         dimensions,
//         pixels. as_slice(),
//     )
// }
//
// fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
//     let image = image::load_from_memory(image_data)?;
//     let size = [image. width() as _, image. height() as _];
//     let image_buffer = image.to_rgba8();
//     let pixels = image_buffer.as_flat_samples();
//     Ok(ColorImage::from_rgba_unmultiplied(
//         size,
//         pixels. as_slice(),
//     ))
// }
//
// fn load_image_from_memory_2(image:DynamicImage) -> Result<ColorImage, image::ImageError> {
//     // let image = image::load_from_memory(image_data)?;
//     // let size = [image. width() as _, image. height() as _];
//     let image_buffer = image.to_rgba8();
//     let pixels = image_buffer.as_flat_samples();
//     Ok(ColorImage::from_rgba_unmultiplied(
//         size,
//         pixels. as_slice(),
//     ))
// }