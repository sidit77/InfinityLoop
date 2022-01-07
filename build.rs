use std::convert::TryInto;
use std::fs;
use std::io::Write;
use anyhow::Result;
use glam::Vec2;
use std::ops::RangeInclusive;
use std::f32::consts::PI;

struct Mesh {
    vertices: Vec<Vec2>,
    indices: Vec<u16>
}

impl Mesh {

    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }

    fn indices_len(&self) -> usize {
        self.indices.len()
    }

    fn add_vertex(&mut self, vertex: Vec2) {
        let index = match self.vertices.iter().position(|v| v.distance(vertex) < 0.001) {
            Some(index) => index,
            None => {
                self.vertices.push(vertex);
                self.vertices.len() - 1
            }
        };
        self.indices.push(index.try_into().unwrap());
    }

    fn draw_arc(&mut self, center: Vec2, radius: f32, angle: RangeInclusive<f32>, thickness: f32, steps: u32) {

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

            self.add_vertex(p_0);
            self.add_vertex(p_1);
            self.add_vertex(p_2);

            self.add_vertex(p_0);
            self.add_vertex(p_2);
            self.add_vertex(p_3);

        }

    }

    fn draw_line(&mut self, v1: Vec2, v2: Vec2, thickness: f32) {
        let extrude = (v2 - v1).normalize().perp() * thickness;

        self.add_vertex(v1 - extrude);
        self.add_vertex(v1 + extrude);
        self.add_vertex(v2 + extrude);

        self.add_vertex(v1 - extrude);
        self.add_vertex(v2 + extrude);
        self.add_vertex(v2 - extrude);
    }

}

fn hexagon_corner(i: u32) -> Vec2 {
    ngon_corner(6, i)
}

fn ngon_corner(n: u32, i: u32) -> Vec2 {
    let (sin, cos) = f32::sin_cos((2.0 / n as f32) * std::f32::consts::PI * i as f32);
    Vec2::new(sin, cos)
}

fn main() -> Result<()> {
    let mut module_file = fs::File::create("./src/meshes.rs")?;
    writeln!(module_file, "#![allow(dead_code)]")?;
    writeln!(module_file, "//This file is autogenerated")?;

    const LINE_THICKNESS: f32 = 0.18;

    let mut mesh = Mesh::new();

    {
        let start = mesh.indices_len();

        mesh.add_vertex(Vec2::new(-1.0, -1.0));
        mesh.add_vertex(Vec2::new(1.0, -1.0));
        mesh.add_vertex(Vec2::new(1.0, 1.0));

        mesh.add_vertex(Vec2::new(-1.0, -1.0));
        mesh.add_vertex(Vec2::new(1.0, 1.0));
        mesh.add_vertex(Vec2::new(-1.0, 1.0));

        writeln!(module_file, "pub const QUAD: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        for i in 0u32..4 {
            mesh.add_vertex(hexagon_corner(0));
            mesh.add_vertex(hexagon_corner(i + 1));
            mesh.add_vertex(hexagon_corner(i + 2));
        }

        writeln!(module_file, "pub const HEXAGON: std::ops::Range<i32> = {}..{};", start,  mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let radius = 0.3;

        let line_1 = (hexagon_corner(0) + hexagon_corner(1)) / 2.0;
        let line_2 = line_1.normalize() * radius;

        mesh.draw_line(line_1, line_2, LINE_THICKNESS);
        mesh.draw_arc(Vec2::ZERO, radius, 0.0..=(2.0 * PI), LINE_THICKNESS, 30);

        writeln!(module_file, "pub const MODEL1: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;
        mesh.draw_arc(hexagon_corner(1), radius, PI..=(1.65 * PI), LINE_THICKNESS, 30);

        writeln!(module_file, "pub const MODEL2: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let line_1 = (hexagon_corner(0) + hexagon_corner(1)) / 2.0;
        let line_2 = (hexagon_corner(3) + hexagon_corner(4)) / 2.0;

        mesh.draw_line(line_1, line_2, LINE_THICKNESS);

        writeln!(module_file, "pub const MODEL3: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        mesh.draw_arc(hexagon_corner(1), radius, PI..=(1.65 * PI), LINE_THICKNESS, 30);
        mesh.draw_arc(hexagon_corner(2), radius, (4.0/3.0 * PI)..=(2.0 * PI), LINE_THICKNESS, 30);

        writeln!(module_file, "pub const MODEL4: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        mesh.draw_arc(hexagon_corner(1), radius, PI..=(1.65 * PI), LINE_THICKNESS, 30);

        mesh.draw_arc(hexagon_corner(4), radius, 0.0..=(2.0 / 3.0 * PI), LINE_THICKNESS, 30);

        mesh.draw_arc(
            hexagon_corner(2) + hexagon_corner(3),
            1.0 + radius, (5.0 / 3.0 * PI)..=(2.0 * PI), LINE_THICKNESS, 30);

        mesh.draw_arc(
            hexagon_corner(5) + hexagon_corner(0),
            1.0 + radius, (2.0 / 3.0 * PI)..=(1.0 * PI), LINE_THICKNESS, 30);

        writeln!(module_file, "pub const MODEL5: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        mesh.draw_arc(
            hexagon_corner(1) + hexagon_corner(2),
            1.0 + radius, ((4.0 / 3.0) * PI)..=((5.0 / 3.0) * PI), LINE_THICKNESS, 30);

        writeln!(module_file, "pub const MODEL6: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    {
        let start = mesh.indices_len();

        let radius = (hexagon_corner(0) - hexagon_corner(1)).length() * 0.5;

        mesh.draw_arc(
            hexagon_corner(3) + hexagon_corner(4),
            1.0 + radius, ((0.0 / 3.0) * PI)..=((1.0 / 3.0) * PI), LINE_THICKNESS, 30);

        mesh.draw_arc(
            hexagon_corner(1) + hexagon_corner(2),
            1.0 + radius, ((4.0 / 3.0) * PI)..=((5.0 / 3.0) * PI), LINE_THICKNESS, 30);

        mesh.draw_arc(
            hexagon_corner(5) + hexagon_corner(0),
            1.0 + radius, ((2.0 / 3.0) * PI)..=((3.0 / 3.0) * PI), LINE_THICKNESS, 30);

        writeln!(module_file, "pub const MODEL7: std::ops::Range<i32> = {}..{};", start, mesh.indices_len())?;
    }

    writeln!(module_file, "pub const VERTICES: &[f32] = &{:?};",
             mesh.vertices.iter().flat_map(|v|[v.x, v.y]).collect::<Vec<_>>())?;
    writeln!(module_file, "pub const INDICES: &[u16] = &{:?};", mesh.indices)?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
