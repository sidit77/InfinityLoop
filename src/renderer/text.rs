use std::collections::HashMap;
use std::ops::Index;
use std::rc::Rc;
use crate::opengl::*;
use anyhow::{Result, ensure};
use artery_font::*;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec2,
    tex_coord: Vec2,
}

impl Vertex {
    fn new(position: impl Into<Vec2>, tex_coord: impl Into<Vec2>) -> Self {
        Self {
            position: position.into(),
            tex_coord: tex_coord.into()
        }
    }
    fn move_by(mut self, offset: impl Into<Vec2>) -> Self {
        self.position += offset.into();
        self
    }
}

#[derive(Debug, Copy, Clone)]
struct Quad {
    plane_bounds: Rect,
    atlas_bounds: Rect
}

impl Quad {

    fn vertices(&self) -> impl Iterator<Item=Vertex> {
        [
            Vertex::new((self.plane_bounds.left , self.plane_bounds.bottom), (self.atlas_bounds.left , self.atlas_bounds.bottom)),
            Vertex::new((self.plane_bounds.right, self.plane_bounds.bottom), (self.atlas_bounds.right, self.atlas_bounds.bottom)),
            Vertex::new((self.plane_bounds.left , self.plane_bounds.top   ), (self.atlas_bounds.left , self.atlas_bounds.top   )),
            Vertex::new((self.plane_bounds.left , self.plane_bounds.top   ), (self.atlas_bounds.left , self.atlas_bounds.top   )),
            Vertex::new((self.plane_bounds.right, self.plane_bounds.bottom), (self.atlas_bounds.right, self.atlas_bounds.bottom)),
            Vertex::new((self.plane_bounds.right, self.plane_bounds.top   ), (self.atlas_bounds.right, self.atlas_bounds.top   )),
        ].into_iter()
    }

}

#[derive(Debug, Copy, Clone)]
struct Glyph {
    advance: f32,
    quad: Option<Quad>
}

impl Glyph {
    fn vertices(&self) -> impl Iterator<Item=Vertex> + '_ {
        self.quad.iter().flat_map(Quad::vertices)
    }
}

#[derive(Debug, Clone)]
struct Line {
    vertices: Vec<Vertex>,
    width: f32
}

#[derive(Debug, Clone)]
struct Text {
    vertices: Vec<Vertex>,
    size: Vec2
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum TextAlignment {
    Left,
    Center,
    Right
}

#[derive(Debug, Clone)]
struct FontInfo {
    metrics: FontMetric,
    glyphs: HashMap<char, Glyph>,
}

impl FontInfo {

    fn px_range(&self) -> f32 {
        self.metrics.distance_range / self.metrics.font_size
    }

    fn line(&self, line: &str) -> Line {
        let mut vertices = Vec::new();
        let mut x= 0.0;

        for glyph in line.chars().map(|c|self.glyphs.index(&c)) {
            for v in glyph.vertices() {
                vertices.push(v.move_by((x,0.0)));
            }
            x += glyph.advance;
        }

        Line {
            vertices,
            width: x
        }
    }

    fn text(&self, text: &str, alignment: TextAlignment) -> Text {
        let mut vertices = Vec::new();
        let lines: Vec<_> = text
            .lines()
            .map(|line| self.line(line))
            .collect();
        let width = lines.iter().map(|line| line.width).fold(0.0, f32::max);
        let mut height = -self.metrics.descender;
        for line in lines.iter().rev() {
            let start = match alignment {
                TextAlignment::Left => 0.0,
                TextAlignment::Center => (width - line.width) * 0.5,
                TextAlignment::Right => width - line.width
            };
            for v in &line.vertices {
                vertices.push(v.move_by((start, height)));
            }
            height += self.metrics.line_height;
        }
        Text { 
            vertices, 
            size: Vec2::new(width, height - self.metrics.line_height + self.metrics.ascender) }
    }

}


pub struct TextBuffer {
    font_info: Rc<FontInfo>,
    vertex_array: VertexArray,
    vertex_buffer: Buffer,
    number_of_vertices: u32,
    size: Vec2
}

impl TextBuffer {
    fn new(ctx: &Context, font_info: Rc<FontInfo>) -> Result<TextBuffer> {
        let vertex_array = VertexArray::new(ctx)?;
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(ctx, BufferTarget::Array)?;
        vertex_array.set_bindings(&vertex_buffer, VertexStepMode::Vertex, &[
            VertexArrayAttribute::Float(0, DataType::F32, 2, false),
            VertexArrayAttribute::Float(1, DataType::F32, 2, false),
        ]);

        Ok(TextBuffer {
            font_info,
            vertex_array,
            vertex_buffer,
            number_of_vertices: 0,
            size: Vec2::ZERO
        })
    }

    pub fn set_text(&mut self, text: &str, alightment: TextAlignment) {
        log::debug!("Changing text to \"{}\"", text);
        let text = self.font_info.text(text, alightment);
        self.number_of_vertices = text.vertices.len() as u32;
        self.vertex_buffer.set_data(&text.vertices, BufferUsage::StaticDraw);
        self.size = text.size;
    }
}

pub struct TextRenderer {
    ctx: Context,
    shader: ShaderProgram,
    texture: Texture,
    font_info: Rc<FontInfo>
}

impl TextRenderer {

    pub fn new(ctx: &Context, font: &ArteryFont, width: u32, height: u32) -> Result<Self> {

        let image = font.images.first().unwrap();
        let variant = font.variants.first().unwrap();
        ensure!(variant.image_type == ImageType::Msdf);
        ensure!(variant.codepoint_type == CodepointType::Unicode);
        //ensure!(variant.kern_pairs.is_empty());
        let metrics = variant.metrics;

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

        let shader = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("../shader/text.vert"))?,
            &Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/text.frag"))?
        ])?;
        ctx.use_program(&shader);
        ctx.set_uniform(&shader.get_uniform("tex")?, 0);

        let mut renderer = Self {
            ctx: ctx.clone(),
            shader,
            texture,
            font_info: Rc::new(FontInfo {
                metrics,
                glyphs
            })
        };
        renderer.resize(ctx, width, height)?;

        Ok(renderer)
    }

    pub fn create_buffer(&self) -> Result<TextBuffer> {
        TextBuffer::new(&self.ctx, self.font_info.clone())
    }

    pub fn render(&self, ctx: &Context, buffer: &TextBuffer) -> Result<()> {
        ensure!(Rc::ptr_eq(&self.font_info, &buffer.font_info));

        if buffer.number_of_vertices > 0 {
            ctx.set_blend_state(BlendState {
                src: BlendFactor::SrcAlpha,
                dst: BlendFactor::OneMinusSrcAlpha,
                equ: BlendEquation::Add
            });
            ctx.use_vertex_array(&buffer.vertex_array);
            ctx.use_program(&self.shader);
            ctx.bind_texture(0, &self.texture);
            ctx.draw_arrays(PrimitiveType::Triangles, 0, buffer.number_of_vertices as i32);
        }
        Ok(())
    }

    pub fn resize(&mut self, ctx: &Context, width: u32, height: u32) -> anyhow::Result<()> {
        let scale = 10.0;
        ctx.use_program(&self.shader);
        ctx.set_uniform(&self.shader.get_uniform("matrix")?, Mat4::orthographic_rh(0.0, (width as f32 / height as f32) * scale, 0.0, scale, 0.0, 1.0));
        ctx.set_uniform(&self.shader.get_uniform("screenPxRange")?, (height as f32 / scale) *  self.font_info.px_range());
        Ok(())
    }

}