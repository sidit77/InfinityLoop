use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;
use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec2};
use crate::{Camera, Color, HexPos};
use crate::opengl::*;
use crate::renderer::TileRenderResources;
use crate::types::Angle;
use crate::util::Apply;
use crate::world::{HexMap, TileConfig, World};

pub struct RenderableWorld {
    resources: Rc<TileRenderResources>,
    vertex_array: VertexArray,
    instance_buffer: Buffer,
    world: World,
    instances: HexMap<RenderState>,
    active_instances: HashSet<HexPos>
}

impl RenderableWorld {

    pub fn new(ctx: &Context, resources: Rc<TileRenderResources>, world: World) -> Result<Self, String> {
        let vertex_array = VertexArray::new(ctx)?;
        ctx.use_vertex_array(&vertex_array);

        let instance_buffer = Buffer::new(ctx, BufferTarget::Array)?;
        vertex_array.set_bindings(&instance_buffer, VertexStepMode::Instance, &[
            VertexArrayAttribute::Float(0, DataType::F32, 3, false),
            VertexArrayAttribute::Float(1, DataType::F32, 3, false),
            VertexArrayAttribute::Float(2, DataType::F32, 3, false),
            VertexArrayAttribute::Integer(3, DataType::U32, 1)
        ]);


        let instances = HexMap::new(world.tiles().radius());

        Ok(Self {
            resources,
            vertex_array,
            instance_buffer,
            world,
            instances,
            active_instances: HashSet::new()
        }.apply(Self::reset))
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

    pub fn render(&self, ctx: &Context, camera: &Camera) {
        ctx.clear(Color::new(0, 0, 0, 255));
        ctx.set_blend_state(BlendState {
            src: BlendFactor::One,
            dst: BlendFactor::One,
            equ: BlendEquation::Max
        });
        self.resources.prepare(ctx, camera);
        ctx.use_vertex_array(&self.vertex_array);
        ctx.draw_arrays_instanced(PrimitiveType::TriangleStrip, 0, 4, self.instances.len() as i32);

    }

    pub fn update(&mut self, delta: Duration) {
        self.active_instances.retain(|pos| self.instances[*pos].update_required());
        for pos in self.active_instances.iter().copied() {
            let offset = self.instances.index(pos).unwrap();
            let instance = &mut self.instances[pos];
            instance.update(delta);
            self.instance_buffer.set_sub_data(offset, &[instance.as_instance()]);
        }

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
                TileConfig::Tile(_, _) => 1.16,
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