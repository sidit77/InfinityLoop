use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;
use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec2};
use crate::{Camera, Color, HexPos};
use crate::opengl::*;
use crate::renderer::TileRenderResources;
use crate::types::Angle;
use crate::util::OptionExt;
use crate::world::{HexMap, TileConfig, World};

pub struct RenderableWorld {
    resources: Rc<TileRenderResources>,
    vertex_array: VertexArray,
    instance_buffer: Buffer,
    framebuffer: Framebuffer,
    framebuffer_dst: Texture,
    last_camera: Option<Camera>,
    world: World,
    instances: HexMap<RenderState>,
    active_instances: HashSet<HexPos>
}

impl RenderableWorld {

    pub fn new(ctx: &Context, resources: Rc<TileRenderResources>, world: World, (width, height): (u32, u32)) -> anyhow::Result<Self> {
        let vertex_array = VertexArray::new(ctx)?;
        ctx.use_vertex_array(&vertex_array);

        let instance_buffer = Buffer::new(ctx, BufferTarget::Array)?;
        vertex_array.set_bindings(&instance_buffer, VertexStepMode::Instance, &[
            VertexArrayAttribute::Float(0, DataType::F32, 3, false),
            VertexArrayAttribute::Float(1, DataType::F32, 3, false),
            VertexArrayAttribute::Float(2, DataType::F32, 3, false),
            VertexArrayAttribute::Integer(3, DataType::U32, 1)
        ]);

        let framebuffer_dst = Texture::new(&ctx, TextureType::Texture2d(width, height), InternalFormat::R8, MipmapLevels::None)?;
        let framebuffer = Framebuffer::new(&ctx, &[
            (FramebufferAttachment::Color(0), &framebuffer_dst)
        ])?;


        let instances = HexMap::new(world.tiles().radius());

        let mut renderer = Self {
            resources,
            vertex_array,
            instance_buffer,
            framebuffer,
            framebuffer_dst,
            last_camera: None,
            world,
            instances,
            active_instances: HashSet::new()
        };
        renderer.reset();
        Ok(renderer)
    }

    pub fn resize(&mut self, ctx: &Context, width: u32, height: u32) -> anyhow::Result<()> {
        self.framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(width, height),
                                            InternalFormat::R8, MipmapLevels::None)?;
        self.framebuffer.update_attachments(&[(FramebufferAttachment::Color(0), &self.framebuffer_dst)])?;
        self.last_camera = None;
        Ok(())
    }

    fn reset(&mut self){
        debug_assert_eq!(self.instances.len(), self.world.tiles().len());
        for (pos, tc) in self.world.iter() {
            self.instances[pos] = RenderState::new(pos, tc);
        }

        let instance_data = self.instances.values().map(RenderState::as_instance).collect::<Vec<Instance>>();
        self.instance_buffer.set_data(instance_data.as_slice(), BufferUsage::DynamicDraw);
    }

    pub fn reinitialize(&mut self, world: World) {
        if self.instances.len() != world.tiles().len() {
            self.instances = HexMap::new(world.tiles().radius());
        }
        self.world = world;
        self.reset()
    }

    pub fn get_texture(&self) -> &Texture {
        &self.framebuffer_dst
    }

    pub fn render(&mut self, ctx: &Context, camera: &Camera) {
        if !self.last_camera.contains_e(camera) {
            ctx.use_framebuffer(&self.framebuffer);
            ctx.clear(Color::new(0, 0, 0, 255));
            ctx.set_blend_state(BlendState {
                src: BlendFactor::One,
                dst: BlendFactor::One,
                equ: BlendEquation::Max
            });
            self.resources.prepare(ctx, camera);
            ctx.use_vertex_array(&self.vertex_array);
            ctx.draw_arrays_instanced(PrimitiveType::TriangleStrip, 0, 4, self.instances.len() as i32);
            self.last_camera = Some(*camera);
            log::trace!("Rerendering")
        }

    }

    pub fn update(&mut self, delta: Duration) {
        self.active_instances.retain(|pos| self.instances[*pos].update_required());
        for pos in self.active_instances.iter().copied() {
            let offset = self.instances.index(pos).unwrap();
            let instance = &mut self.instances[pos];
            instance.update(delta);
            self.instance_buffer.set_sub_data(offset, &[instance.as_instance()]);
            self.last_camera = None;
        }

    }

    pub fn update_required(&self) -> bool {
        !self.active_instances.is_empty() || self.last_camera.is_none()
    }

    pub fn try_rotate(&mut self, pos: HexPos) -> bool {
        let result = self.world.try_rotate(pos);
        if result {
            let tc = self.world.tiles()[pos];
            self.instances[pos].update_target_rotation(tc.angle());

            self.active_instances.insert(pos);
        }

        result
    }

    pub fn is_completed(&self) -> bool {
        self.world.is_completed()
    }

    pub fn seed(&self) -> u64 {
        self.world.seed()
    }

}

impl From<RenderableWorld> for World {
    fn from(renderer: RenderableWorld) -> Self {
        renderer.world
    }
}

#[derive(Debug, Copy, Clone, Default, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    model: Mat3,
    texture: u32
}

#[derive(Debug, Copy, Clone, Default)]
struct RenderState {
    pos: Vec2,
    scale: f32,
    texture: u32,
    current_rotation: Angle,
    target_rotation: Angle
}

impl RenderState {

    fn new(pos: HexPos, config: TileConfig) -> Self {
        Self {
            pos: pos.into(),
            scale: match config {
                TileConfig::Empty => 0.0,
                TileConfig::Tile(_, _) => 1.155,
            },
            texture: config.model() as u32,
            current_rotation: config.angle(),
            target_rotation: config.angle()
        }
    }

    fn as_instance(&self) -> Instance {
        Instance {
            model: Mat3::from_scale_angle_translation(
                Vec2::ONE * self.scale,
                self.current_rotation.to_radians(),
                self.pos
            ),
            texture: self.texture
        }
    }

    fn update(&mut self, delta: Duration) {
        self.current_rotation = Angle::lerp_snap(self.current_rotation, self.target_rotation,
                                                 1.0 - f32::exp(-14.0 * delta.as_secs_f32()),
                                                 Angle::radians(0.03));
    }

    fn update_required(&self) -> bool {
        self.current_rotation != self.target_rotation
    }

    fn update_target_rotation(&mut self, target: Angle) {
        self.target_rotation = target;
    }

}