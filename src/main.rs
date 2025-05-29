use ico::{IconDir, IconImage};
use image::{DynamicImage, GenericImageView};
use resvg::tiny_skia::Pixmap;
use resvg::tiny_skia::Transform;
use resvg::FitTo;
use std::fs;
use std::path::{Path, PathBuf};
use usvg::TreeParsing;
use usvg::{Options, Tree};
use walkdir::WalkDir;
const MAX_DIMENSION: u32 = 256;

fn clear_dir(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(path)?;
            }
        }
    }
    Ok(())
}

fn render_svg_to_png(svg_path: &Path, png_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let svg_data = std::fs::read(svg_path)?;
    let options = Options::default();
    let rtree = Tree::from_data(&svg_data, &options)?;
    let svg_size = rtree.size;
    let scale = (MAX_DIMENSION as f64 / svg_size.width())
        .min(MAX_DIMENSION as f64 / svg_size.height()) as f64;
    let target_width = (svg_size.width() * scale).ceil() as u32;
    let target_height = (svg_size.height() * scale).ceil() as u32;
    let mut pixmap = Pixmap::new(target_width, target_height).ok_or("Failed to create pixmap")?;
    resvg::render(
        &rtree,
        FitTo::Zoom(scale as f32),
        Transform::identity(),
        pixmap.as_mut(),
    )
    .ok_or("Failed to render SVG")?;
    let img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(target_width, target_height, pixmap.data().to_vec())
            .ok_or("Failed to convert pixmap to image buffer")?,
    );
    let mut square_img = image::RgbaImage::new(MAX_DIMENSION, MAX_DIMENSION);
    for pixel in square_img.pixels_mut() {
        *pixel = image::Rgba([0, 0, 0, 0]);
    }
    let x_offset = (MAX_DIMENSION - target_width) / 2;
    let y_offset = (MAX_DIMENSION - target_height) / 2;
    image::imageops::overlay(&mut square_img, &img, x_offset.into(), y_offset.into());
    square_img.save(png_path)?;
    Ok(())
}

fn convert_png_to_ico(png_path: &Path, ico_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(png_path)?;
    let (width, height) = img.dimensions();
    let icon_img = IconImage::from_rgba_data(width, height, img.to_rgba8().into_raw());
    let entry = ico::IconDirEntry::encode(&icon_img)?;
    let mut icon_dir = IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(entry);
    let mut file = std::fs::File::create(ico_path)?;
    icon_dir.write(&mut file)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_dir = Path::new("./data/SVG");
    let png_dir = Path::new("./data/PNG");
    let ico_dir = Path::new("./data/ICO");
    fs::create_dir_all(png_dir)?;
    fs::create_dir_all(ico_dir)?;
    clear_dir(png_dir)?;
    clear_dir(ico_dir)?;
    for entry in WalkDir::new(svg_dir).min_depth(1).max_depth(1) {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("svg"))
        {
            let file_stem = path.file_stem().unwrap().to_string_lossy();
            let png_path = png_dir.join(format!("{}.png", file_stem));
            let ico_path = ico_dir.join(format!("{}.ico", file_stem));
            println!("Processing: {}", path.display());
            render_svg_to_png(path, &png_path)?;
            convert_png_to_ico(&png_path, &ico_path)?;
        }
    }
    println!("Done!");
    Ok(())
}
