use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Luma};
use std::{
    cmp,
    env::{self, args},
    io::Cursor,
};
/* Quick Routine of the Program
* 1. Parse Arguments given by the CLI
* 2. Load Image file!()
* 3. Apply ASCII "remap"
* 4. output in the console (can be used in combination of '>> "out.txt"')
* */

const ASCII_CHARS: &[u8] = b" .:-=+*#%@";

#[derive(Debug)]
struct CliArgs {
    file: String,
    max_width: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let img = ImageReader::open(args.file)?.decode()?;

    let ascii_image = image_to_ascii(img, args.max_width);

    println!("{}", ascii_image);

    Ok(())
}

/// Crop `img` to the largest centered 16:9 region.
fn crop_to_16_9(img: &DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let target_w16 = h * 16 / 9;
    let target_h9 = w * 9 / 16;

    if target_w16 <= w {
        // height is the limiting factor: crop width to h*(16/9)
        let x0 = (w - target_w16) / 2;
        img.crop_imm(x0, 0, target_w16, h)
    } else {
        // width is the limiting factor: crop height to w*(9/16)
        let y0 = (h - target_h9) / 2;
        img.crop_imm(0, y0, w, target_h9)
    }
}

/// Convert an image to ASCII, cropping to 16:9 and resizing to `max_width` columns.
fn image_to_ascii(img: DynamicImage, max_width: u32) -> String {
    // 1) Crop to 16:9
    let cropped = crop_to_16_9(&img);

    // 2) Decide output width
    let out_w = cmp::min(cropped.width(), max_width);

    // 3) Compute output height in characters.
    //    true 16:9 for characters means: height_chars / width_chars = 9/16,
    //    but we also correct for character cell aspect (height / width).
    let char_aspect = 0.5_f32;
    let out_h = ((out_w as f32 * 9.0 / 16.0) * char_aspect).round().max(1.0) as u32;

    // 4) Resize & to grayscale
    let gray = cropped
        .resize_exact(out_w, out_h, image::imageops::FilterType::Nearest)
        .to_luma8();

    // 5) Build ASCII
    let (w, h) = gray.dimensions();
    let mut buf = String::with_capacity((w as usize + 1) * h as usize);
    for y in 0..h {
        for x in 0..w {
            let Luma([l]) = gray.get_pixel(x, y);
            let idx = (*l as usize * (ASCII_CHARS.len() - 1)) / 255;
            buf.push(ASCII_CHARS[idx] as char);
        }
        buf.push('\n');
    }
    buf
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 1 {
        print_usage();
        std::process::exit(1);
    }

    let standard_width = 160;
    let m_width: u32 = match args.get(1) {
        Some(s) => match s.parse() {
            Ok(v) => v,
            Err(e) => {
                print_usage();
                println!("{}", e);
                println!("Using standard width: {}", standard_width);
                standard_width
            }
        },
        None => standard_width,
    };

    CliArgs {
        file: args[0].clone(),
        max_width: m_width,
    }
}
fn print_usage() {
    println!("Usage: image-to-ascii file.png max_width")
}
