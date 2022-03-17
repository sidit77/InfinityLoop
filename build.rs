use std::env;
use std::path::Path;
use glam::Vec2;
use image::{GrayImage, Luma};
use anyhow::Result;
use sdf2d::{Sdf, Shapes, Ops, Constant};

const  WIDTH: u32 = 64;
const  HEIGHT: u32 = 64;
const  FACTOR: f32 = -5.0;

fn main() -> Result<()> {
    let out_dir = Path::new(&env::var("OUT_DIR")?).to_owned();

    let a = 0.75;
    let g = 0.75 * f32::tan(f32::to_radians(30.0));

    let hexagon = Shapes::hexagon(a).rotate(f32::to_radians(90.0));
    rasterize(hexagon).save(out_dir.join("hex.png"))?;

    let tile0 = Shapes::circle(0.45)
        .subtract(Shapes::circle(0.25))
        .union(Shapes::rectangle(0.1, 0.25)
            .translate(0.0, -0.5)
            .rotate(f32::to_radians(30.0)));
    rasterize(tile0).save(out_dir.join("tile0.png"))?;

    let tile01 = Shapes::circle(g + 0.1)
        .subtract(Shapes::circle(g - 0.1))
        .translate(a, -g)
        .intersection(hexagon);
    rasterize(tile01).save(out_dir.join("tile01.png"))?;

    let tile02 = Shapes::circle(3.0 * g + 0.1)
        .subtract(Shapes::circle(3.0 * g - 0.1))
        .translate(2.0 * a, 0.0)
        .intersection(hexagon);
    rasterize(tile02).save(out_dir.join("tile02.png"))?;

    let tile03 = Shapes::rectangle(0.1, 0.75)
        .rotate(f32::to_radians(210.0));
    rasterize(tile03).save(out_dir.join("tile03.png"))?;

    let tile012 = Constant::Empty
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(a, -g))
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(a,  g))
        .intersection(hexagon);
    rasterize(tile012).save(out_dir.join("tile012.png"))?;

    let tile024 = Constant::Empty
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(2.0 * a, 0.0))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(-a, 3.0 * g))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(-a, -3.0 * g))
        .intersection(hexagon);
    rasterize(tile024).save(out_dir.join("tile024.png"))?;

    let tile0134 = Constant::Empty
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(a, -g))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(a, 3.0 * g))
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(-a, g))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(-a, -3.0 * g))
        .intersection(hexagon);
    rasterize(tile0134).save(out_dir.join("tile0134.png"))?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn rasterize(sdf: impl Sdf) -> GrayImage {
    let f = Vec2::new(WIDTH as f32, HEIGHT as f32) * 0.5;
    let img = GrayImage::from_fn(WIDTH, HEIGHT, |x, y| {
        let p = (Vec2::new(x as f32, y as f32) + Vec2::new(0.5, 0.5) - f) / f;
        let d = FACTOR * sdf.density(p);
        let h = u8::MAX as f32 * 0.5;
        Luma([(h + d * h).clamp(u8::MIN as f32, u8::MAX as f32) as u8])
    });
    image::imageops::flip_vertical(&img)
}
