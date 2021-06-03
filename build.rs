
use std::env;
use std::fs;
use std::path::Path;
use std::io::Write;
use anyhow::Result;
use glam::Vec2;

fn main() -> Result<()> {
    let mesh_path = "meshes.bin";
    let mut mesh_file = fs::File::create(Path::new(env::var("OUT_DIR")?.as_str()).join(mesh_path))?;
    let mut module_file = fs::File::create("./src/meshes.rs")?;
    writeln!(module_file, "#![allow(dead_code)]")?;
    writeln!(module_file, "//This file is autogenerated")?;
    writeln!(module_file, "pub const VERTEX_DATA: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{}\"));", mesh_path)?;

    const LINE_THICKNESS: f32 = 0.2;

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

        let radius = 0.4;
        let segments = 32;

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


        for i in 0..segments {
            let p_0 = ngon_corner(segments, i + 0) * (radius + LINE_THICKNESS * 0.5);
            let p_1 = ngon_corner(segments, i + 1) * (radius + LINE_THICKNESS * 0.5);
            let p_2 = ngon_corner(segments, i + 1) * (radius - LINE_THICKNESS * 0.5);
            let p_3 = ngon_corner(segments, i + 0) * (radius - LINE_THICKNESS * 0.5);

            mesh_file.write_all(bytemuck::bytes_of(&p_0))?;
            mesh_file.write_all(bytemuck::bytes_of(&p_1))?;
            mesh_file.write_all(bytemuck::bytes_of(&p_2))?;
            index += 3;

            mesh_file.write_all(bytemuck::bytes_of(&p_0))?;
            mesh_file.write_all(bytemuck::bytes_of(&p_2))?;
            mesh_file.write_all(bytemuck::bytes_of(&p_3))?;
            index += 3;
        }

        writeln!(module_file, "pub const MODEL1: std::ops::Range<i32> = {}..{};", start, index)?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn hexagon_corner(i: u32) -> Vec2 {
    ngon_corner(6, i)
}

fn ngon_corner(n: u32, i: u32) -> Vec2 {
    let (sin, cos) = f32::sin_cos((2.0 / n as f32) * std::f32::consts::PI * i as f32);
    Vec2::new(sin, cos)
}