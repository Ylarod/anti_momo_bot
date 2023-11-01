use std::collections::HashMap;
use color_space::{CompareCie2000, Rgb};
use image::{DynamicImage, GenericImageView, Pixel};
use rusty_tesseract::{Args, Image};
use crate::defs;

const THRESHOLD: f64 = 10.0;

const MOMO_RGB: Rgb = Rgb{
    r: 146.0,
    g: 74.0,
    b: 96.0
};

pub fn is_momo_screenshot(path: &str) -> anyhow::Result<bool> {
    let img = image::open(path)?;
    let rgb = get_most_frequent_color(&img);
    let diff = MOMO_RGB.compare_cie2000(&rgb);
    let mut result = diff < THRESHOLD;
    if unsafe { defs::USE_OCR } {
        result = result && ocr_find_momo(&img)?;
    }
    return Ok(result);
}

pub fn ocr_find_momo(img: &DynamicImage) -> anyhow::Result<bool> {
    let img = Image::from_dynamic_image(img)?;
    let args = Args {
        lang: "eng".to_string(),
        config_variables: HashMap::from([(
            "tessedit_char_whitelist".into(),
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".into(),
        )]),
        dpi: None,
        psm: None,
        oem: None,
    };
    let output = rusty_tesseract::image_to_string(&img, &args)?;
    println!("The String output is: {:?}", output);
    if output.find("Momo").is_some() {
        return Ok(true)
    }else if output.find("momo").is_some() {
        return Ok(true)
    }
    Ok(false)
}

fn get_most_frequent_color(img: &DynamicImage) -> Rgb {
    let width = img.width();
    let height = img.height();
    let mut color_counts: HashMap<image::Rgb<u8>, u64> = HashMap::new();
    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            let rgb = pixel.to_rgb();
            *color_counts.entry(rgb).or_insert(0) += 1;
        }
    }
    let color_count: Vec<(Rgb, u64)> = color_counts
        .iter()
        .map(|(color, count)| (Rgb::new(color[0] as f64, color[1] as f64,color[2] as f64), *count))
        .collect();
    let result = color_count.iter().max_by(|(_, a), (_, b)| a.cmp(b)).unwrap();
    return result.0;
}

#[cfg(test)]
mod tests {
    use crate::momo::is_momo_screenshot;

    #[test]
    fn momo_judge() {
        let start_time = std::time::Instant::now();
        let p1 = is_momo_screenshot("res/pic1.png").unwrap();
        let p2 = is_momo_screenshot("res/pic2.jpg").unwrap();
        let elapsed_time = start_time.elapsed().as_millis();
        println!("momo judge: {p1} {p2} cost {elapsed_time}");
    }
}