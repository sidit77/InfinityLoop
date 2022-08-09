use std::collections::HashMap;
use crate::opengl::*;
use anyhow::{Result, ensure};
use artery_font::*;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4};
use crate::AppContext;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

#[derive(Debug, Copy, Clone)]
struct Quad {
    plane_bounds: Rect,
    atlas_bounds: Rect
}

impl Quad {

    fn vertices(&self, x_offset: f32, y_offset: f32) -> impl Iterator<Item=Vertex> {
        [
            Vertex { position: [x_offset + self.plane_bounds.left , y_offset + self.plane_bounds.bottom], tex_coord: [self.atlas_bounds.left , self.atlas_bounds.bottom] },
            Vertex { position: [x_offset + self.plane_bounds.right, y_offset + self.plane_bounds.bottom], tex_coord: [self.atlas_bounds.right, self.atlas_bounds.bottom] },
            Vertex { position: [x_offset + self.plane_bounds.left , y_offset + self.plane_bounds.top   ], tex_coord: [self.atlas_bounds.left , self.atlas_bounds.top   ] },
            Vertex { position: [x_offset + self.plane_bounds.left , y_offset + self.plane_bounds.top   ], tex_coord: [self.atlas_bounds.left , self.atlas_bounds.top   ] },
            Vertex { position: [x_offset + self.plane_bounds.right, y_offset + self.plane_bounds.bottom], tex_coord: [self.atlas_bounds.right, self.atlas_bounds.bottom] },
            Vertex { position: [x_offset + self.plane_bounds.right, y_offset + self.plane_bounds.top   ], tex_coord: [self.atlas_bounds.right, self.atlas_bounds.top   ] },
        ].into_iter()
    }

}

#[derive(Debug, Copy, Clone)]
struct Glyph {
    advance: f32,
    quad: Option<Quad>
}

impl Glyph {

    fn vertices(&self, x_offset: f32, y_offset: f32) -> impl Iterator<Item=Vertex> + '_ {
        self.quad.iter().flat_map(move |q|q.vertices(x_offset, y_offset))
    }

}

pub struct TextRenderer{
    vertex_array: VertexArray,
    vertex_buffer: Buffer,
    shader: ShaderProgram,
    texture: Texture,
    line_height: f32,
    pxrange: f32,
    glyphs: HashMap<char, Glyph>,
    number_of_vertices: u32
}

impl TextRenderer {

    pub fn new(ctx: &Context, font: &ArteryFont) -> Result<Self> {

        let image = font.images.first().unwrap();
        let variant = font.variants.first().unwrap();
        ensure!(variant.image_type == ImageType::Msdf);
        ensure!(variant.codepoint_type == CodepointType::Unicode);
        ensure!(variant.kern_pairs.is_empty());
        let line_height = variant.metrics.line_height;
        let pxrange = variant.metrics.distance_range / variant.metrics.font_size;

        let mut glyphs = HashMap::new();

        for glyph in &variant.glyphs {
            let unicode = std::char::from_u32(glyph.codepoint).unwrap();
            let advance = glyph.advance.horizontal;
            ensure!(glyph.advance.vertical == 0.0, "character: {}", unicode);
            ensure!(glyph.image == 0);
            glyphs.insert(unicode, Glyph {
                advance,
                quad: match glyph.is_drawable() {
                    true => Some(Quad {
                        plane_bounds: glyph.plane_bounds,
                        atlas_bounds: glyph.image_bounds.scaled(1.0 / image.width as f32, 1.0 / image.height as f32)
                    }),
                    false => None
                }
            });
        }

        ensure!(image.channels == 3);
        ensure!(image.pixel_format == PixelFormat::Unsigned8);

        let texture = Texture::new(ctx, TextureType::Texture2d(image.width, image.height), InternalFormat::Rgb8, MipmapLevels::Full)?;
        texture.fill_region_2d(0, Region2d::dimensions(image.width, image.height),Format::Rgb, DataType::U8, &image.data);
        texture.generate_mipmaps();
        texture.set_wrap_mode(WrapMode {
            s: TextureWrap::ClampToEdge,
            t: TextureWrap::ClampToEdge,
            r: TextureWrap::ClampToEdge
        });

        let vertex_array = VertexArray::new(ctx)?;
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(ctx, BufferTarget::Array)?;
        vertex_array.set_bindings(&vertex_buffer, VertexStepMode::Vertex, &[
            VertexArrayAttribute::Float(0, DataType::F32, 2, false),
            VertexArrayAttribute::Float(1, DataType::F32, 2, false),
        ]);

        let shader = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("../shader/text.vert"))?,
            &Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/text.frag"))?
        ])?;
        ctx.use_program(&shader);
        ctx.set_uniform(&shader.get_uniform("tex")?, 0);

        Ok(Self {
            vertex_array,
            vertex_buffer,
            shader,
            texture,
            line_height,
            pxrange,
            glyphs,
            number_of_vertices: 0
        })
    }

    pub fn set_text(&mut self, text: &str) {
        log::debug!("Changing text to \"{}\"", text);
        let mut vertices = Vec::new();
        let mut x;
        let mut y = 0.2;

        for line in text.lines() {
            x = 0.2;
            for glyph in line.chars().map(|c|self.glyphs[&c]) {
                for v in glyph.vertices(x, y) {
                    vertices.push(v);
                }
                x += glyph.advance;
            }
            y -= self.line_height * 0.55;
        }

        self.number_of_vertices = vertices.len() as u32;
        self.vertex_buffer.set_data(&vertices, BufferUsage::StaticDraw);
    }

    pub fn render<A: AppContext>(&self, ctx: &A) -> Result<()> {
        let (width, height) = ctx.screen_size();
        let scale = 10.0;
        if self.number_of_vertices > 0 {
            ctx.set_blend_state(BlendState {
                src: BlendFactor::SrcAlpha,
                dst: BlendFactor::OneMinusSrcAlpha,
                equ: BlendEquation::Add
            });
            ctx.use_vertex_array(&self.vertex_array);
            ctx.use_program(&self.shader);
            ctx.set_uniform(&self.shader.get_uniform("matrix")?, Mat4::orthographic_rh(0.0, (width as f32 / height as f32) * scale, 0.0, scale, 0.0, 1.0));
            ctx.set_uniform(&self.shader.get_uniform("screenPxRange")?, (height as f32 / scale) * self.pxrange);
            ctx.bind_texture(0, &self.texture);
            ctx.draw_arrays(PrimitiveType::Triangles, 0, self.number_of_vertices as i32);
        }
        Ok(())
    }

}