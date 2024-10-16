#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ConversionInput {
    fahrenheit: f64,
}

#[derive(Serialize)]
struct ConversionOutput {
    celsius: f64,
}

// Convert Fahrenheit to Celsius
fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

#[get("/convert?<fahrenheit>")]
fn convert(fahrenheit: f64) -> Json<ConversionOutput> {
    let celsius = fahrenheit_to_celsius(fahrenheit);
    Json(ConversionOutput { celsius })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![convert])
        .mount("/", rocket::fs::FileServer::from("static"))
}
