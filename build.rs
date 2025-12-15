use anyhow::Result;
use image::DynamicImage;
use image::ImageFormat;
use image::ImageReader;
use std::env::var;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use vergen_gitcl::{Emitter, GitclBuilder};

#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn configure_winres(ico: &Path) {
    let mut res = winres::WindowsResource::new();
    res.set_icon(ico.to_str().unwrap());
    res.compile().unwrap();
}

#[cfg(unix)]
fn configure_winres(_ico: &Path) {}

fn load_icon() -> DynamicImage {
    let png: &[u8] = include_bytes!("assets/logo.png");
    let mut reader = ImageReader::new(Cursor::new(png));
    reader.set_format(ImageFormat::Png);
    reader.decode().unwrap()
}

fn create_icon_bin(icon: &DynamicImage) {
    let bin_path = PathBuf::from(var("OUT_DIR").unwrap()).join("icon.bin");
    let width = 64;
    let height = 64;

    println!("cargo:rustc-env=ICO_BIN_PATH={}", bin_path.display());
    println!("cargo:rustc-env=ICO_BIN_WIDTH={}", width);
    println!("cargo:rustc-env=ICO_BIN_HEIGHT={}", height);

    let mut file = File::create(bin_path).unwrap();
    file.write_all(
        icon.resize(width, height, image::imageops::FilterType::Nearest)
            .as_bytes(),
    )
    .unwrap();
}

fn create_icon(icon: &DynamicImage) -> PathBuf {
    let ico_path = PathBuf::from(var("OUT_DIR").unwrap()).join("icon.ico");
    icon.resize(64, 64, image::imageops::FilterType::Nearest)
        .save_with_format(&ico_path, ImageFormat::Ico)
        .unwrap();
    ico_path
}

pub fn main() -> Result<()> {
    Emitter::default()
        .add_instructions(&GitclBuilder::all_git()?)?
        .emit()?;

    let icon = load_icon();
    create_icon_bin(&icon);
    configure_winres(&create_icon(&icon));

    Ok(())
}
