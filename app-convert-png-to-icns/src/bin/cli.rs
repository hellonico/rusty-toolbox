use app_convert_png_to_icns::png_to_icns;
use std::path::Path;

fn main(){
    let input = std::env::args().nth(1).unwrap();
    let output = Path::new(&input.clone())
        .with_extension("icns")
        .to_str()
        .unwrap()
        .to_string();
    // let output_path = Some(output.clone());

    if let Err(e) = png_to_icns(&input.clone(), &output) {
        eprintln!("{}", e);
    } else {
        println!("Converted icns into png to {}", output);
    }
}