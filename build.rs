
use std::env;
use std::fs;
use std::path::Path;
use std::io::Write;
use anyhow::Result;
use glam::Vec2;
use std::ops::RangeInclusive;
use std::f32::consts::PI;

fn main() -> Result<()> {
    let mesh_path = "meshes.bin";
    let mut mesh_file = fs::File::create(Path::new(env::var("OUT_DIR")?.as_str()).join(mesh_path))?;
    let mut module_file = fs::File::create("./src/meshes.rs")?;
    writeln!(module_file, "#![allow(dead_code)]")?;
    writeln!(module_file, "//This file is autogenerated")?;
    writeln!(module_file, "pub const VERTEX_DATA: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{}\"));", mesh_path)?;

    const LINE_THICKNESS: f32 = 0.14;

    let mut index = 0;

    {
        let start = index;

        for i in 0u32..4 {
            mesh_file.write_all(bytemuck::bytes_of(&hexagon_corner(0)))?;
            mesh_file.write_all(bytemuck::bytes_of(&hexagon_corner(i + 1)))?;
            mesh_file.write_all(bytemuck::bytes_of(&hexagon_corner(i + 2)))?;
            index += 3;
        }

        writeln!(module_file, "pub const HEXAGON: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let radius = 0.3;

        let line_1 = (hexagon_corner(0) + hexagon_corner(1)) / 2.0;
        let line_2 = line_1.normalize() * radius;
        let extrude = (hexagon_corner(0) - hexagon_corner(1)).normalize() * (LINE_THICKNESS * 0.5);

        mesh_file.write_all(bytemuck::bytes_of(&(line_1 - extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_1 + extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_2 + extrude)))?;
        index += 3;

        mesh_file.write_all(bytemuck::bytes_of(&(line_1 - extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_2 + extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_2 - extrude)))?;
        index += 3;


        let circle = draw_arc(Vec2::ZERO, radius, 0.0..=(2.0 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        writeln!(module_file, "pub const MODEL1: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        let circle = draw_arc(hexagon_corner(1), radius, PI..=(1.65 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        writeln!(module_file, "pub const MODEL2: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let line_1 = (hexagon_corner(0) + hexagon_corner(1)) / 2.0;
        let line_2 = (hexagon_corner(3) + hexagon_corner(4)) / 2.0;
        let extrude = (hexagon_corner(0) - hexagon_corner(1)).normalize() * (LINE_THICKNESS * 0.5);

        mesh_file.write_all(bytemuck::bytes_of(&(line_1 - extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_1 + extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_2 + extrude)))?;
        index += 3;

        mesh_file.write_all(bytemuck::bytes_of(&(line_1 - extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_2 + extrude)))?;
        mesh_file.write_all(bytemuck::bytes_of(&(line_2 - extrude)))?;
        index += 3;

        writeln!(module_file, "pub const MODEL3: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        let circle = draw_arc(hexagon_corner(1), radius, PI..=(1.65 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        let circle = draw_arc(hexagon_corner(2), radius, (4.0/3.0 * PI)..=(2.0 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        writeln!(module_file, "pub const MODEL4: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        let circle = draw_arc(hexagon_corner(1), radius, PI..=(1.65 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        let circle = draw_arc(hexagon_corner(4), radius, 0.0..=(2.0 / 3.0 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        let circle = draw_arc(
            hexagon_corner(2) + hexagon_corner(3),
            1.0 + radius, (5.0 / 3.0 * PI)..=(2.0 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        let circle = draw_arc(
            hexagon_corner(5) + hexagon_corner(0),
            1.0 + radius, (2.0 / 3.0 * PI)..=(1.0 * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        writeln!(module_file, "pub const MODEL5: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        let circle = draw_arc(
            hexagon_corner(1) + hexagon_corner(2),
            1.0 + radius, ((4.0 / 3.0) * PI)..=((5.0 / 3.0) * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        writeln!(module_file, "pub const MODEL6: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    {
        let start = index;

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        let circle = draw_arc(
            hexagon_corner(3) + hexagon_corner(4),
            1.0 + radius, ((0.0 / 3.0) * PI)..=((1.0 / 3.0) * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        let circle = draw_arc(
            hexagon_corner(1) + hexagon_corner(2),
            1.0 + radius, ((4.0 / 3.0) * PI)..=((5.0 / 3.0) * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        let circle = draw_arc(
            hexagon_corner(5) + hexagon_corner(0),
            1.0 + radius, ((2.0 / 3.0) * PI)..=((3.0 / 3.0) * PI), LINE_THICKNESS, 30);
        index += circle.len();
        mesh_file.write_all(bytemuck::cast_slice(circle.as_slice()))?;

        writeln!(module_file, "pub const MODEL7: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn draw_arc(center: Vec2, radius: f32, angle: RangeInclusive<f32>, thickness: f32, steps: u32) -> Vec<Vec2> {
    let mut result = Vec::new();

    let pos = | i | {
        if i == steps {
            *angle.end()
        } else {
            angle.start() + (angle.end() - angle.start()) * (i as f32 / steps as f32)
        }
    };

    for i in 0..=steps {
        let p_0 = center + Vec2::from(f32::sin_cos(pos(i + 0))) * (radius + thickness * 0.5);
        let p_1 = center + Vec2::from(f32::sin_cos(pos(i + 1))) * (radius + thickness * 0.5);
        let p_2 = center + Vec2::from(f32::sin_cos(pos(i + 1))) * (radius - thickness * 0.5);
        let p_3 = center + Vec2::from(f32::sin_cos(pos(i + 0))) * (radius - thickness * 0.5);

        result.push(p_0);
        result.push(p_1);
        result.push(p_2);

        result.push(p_0);
        result.push(p_2);
        result.push(p_3);

    }

    result
}

fn hexagon_corner(i: u32) -> Vec2 {
    ngon_corner(6, i)
}

fn ngon_corner(n: u32, i: u32) -> Vec2 {
    let (sin, cos) = f32::sin_cos((2.0 / n as f32) * std::f32::consts::PI * i as f32);
    Vec2::new(sin, cos)
}