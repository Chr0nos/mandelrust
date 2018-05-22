extern crate image;
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;
//use std::io::Write;
extern crate num;
use num::Complex;
extern crate sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
extern crate rayon;
use rayon::prelude::*;

struct Tv2i {
    x: usize,
    y: usize
}

struct Mandelrust {
    dimentions: Tv2i,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
    threads: usize,
    max_iterations: u32
}

impl Mandelrust {
    pub fn render(&self, pixels: &mut [u8])
    {
        assert!(pixels.len() == self.dimentions.x * self.dimentions.y);
        let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(self.dimentions.x)
            .enumerate().collect();
        bands.into_par_iter().weight_max().for_each(|(i, band)| {
            let top = i;
            let band_bounds = Tv2i { x: self.dimentions.x, y: 1 };
            let band_upper_left = self.pixel_to_point(
                Tv2i { x: 0, y: top },
                self.upper_left,
                self.lower_right,
                &self.dimentions);
            let band_lower_right = self.pixel_to_point(
                Tv2i { x: self.dimentions.x, y: top + 1 },
                self.upper_left,
                self.lower_right,
                &self.dimentions);
            self.render_section(band, band_bounds, band_upper_left, band_lower_right);
        });
    }

    fn pixel_to_point(&self, pixel: Tv2i,
                      upper_left: Complex<f64>,
                      lower_right: Complex<f64>,
                      geometry: &Tv2i) -> Complex<f64>
    {
        let (width, height) = (
            lower_right.re - upper_left.re,
            upper_left.im - lower_right.im
        );
        Complex {
            re: upper_left.re + pixel.x as f64 * width / geometry.x as f64,
            im: upper_left.im - pixel.y as f64 * height / geometry.y as f64
        }
    }

    fn render_section(&self, pixels: &mut [u8], bounds: Tv2i,
                     upper_left: Complex<f64>, lower_right: Complex<f64>) {
        for row in 0 .. bounds.y {
            for column in 0 .. bounds.x {
                let point = self.pixel_to_point(Tv2i {x: column, y: row},
                                                upper_left, lower_right, &bounds);
                pixels[row * bounds.x + column] = match self.compute(point) {
                    None => 0,
                    Some(value) => 255 - value as u8
                };
            }
        }
    }

    fn compute(&self, c: Complex<f64>) -> Option<u32> {
        let mut z = Complex { re: 0.0, im: 0.0 };
        for i in 0..self.max_iterations {
            z = z * z + c;
            if z.norm_sqr() > 4.0 {
                return Some(i);
            }
        }
        None
    }
}

fn write_image(filename: &str, pixels: &[u8], geometry: &Tv2i) -> Result<(), std::io::Error>
{
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels, geometry.x as u32, geometry.y as u32, ColorType::Gray(8))?;
    Ok(())
}

fn window(mand: &Mandelrust)
{
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window(
        "Mandelrust", mand.dimentions.x as u32, mand.dimentions.y as u32)
        .position_centered().build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    let mut pixels = vec![0; mand.dimentions.x * mand.dimentions.y];
    let surface = Surface::from_data(&mut pixels,
        mand.dimentions.x as u32, mand.dimentions.y as u32, 4,
        PixelFormatEnum::RGBA4444);
    canvas.present();
}

fn main() {
    let mand: Mandelrust = Mandelrust {
        dimentions: Tv2i {x: 1920, y: 1080},
        upper_left: Complex { re: -1.20, im: 0.35},
        lower_right: Complex { re: -1.0, im: 0.20},
        threads: 12,
        max_iterations: 255
    };

    assert!(mand.threads > 0);
    assert!(mand.max_iterations > 0);
    assert!(mand.dimentions.x * mand.dimentions.y > 0);

    let mut pixels = vec![0; mand.dimentions.x * mand.dimentions.y];
    mand.render(&mut pixels);
    window(&mand);
    write_image("test.png", &pixels, &mand.dimentions)
        .expect("failed to write file");
}
