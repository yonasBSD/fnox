// Build script for fnox
// Generates settings code from settings.toml

#[path = "build/generate_settings.rs"]
mod generate_settings;

fn main() {
    // Tell Cargo to rerun this build script if settings.toml changes
    println!("cargo:rerun-if-changed=settings.toml");

    // Generate settings code
    generate_settings::generate().expect("Failed to generate settings code");
}
