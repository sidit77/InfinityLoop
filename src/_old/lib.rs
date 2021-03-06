impl Game for InfinityLoop {
    fn initialize(ctx: &Context) -> Self {

        let pp_program = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("shader/postprocess.vert")).unwrap(),
            &Shader::new(ctx, ShaderType::Fragment, include_str!("shader/postprocess.frag")).unwrap(),
        ]).unwrap();
        ctx.use_program(&pp_program);
        ctx.set_uniform(&pp_program.get_uniform("tex").unwrap(), 0);

        let screen_size = PhysicalSize::new(1280, 720);
        let framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(screen_size.width, screen_size.height), InternalFormat::R8, MipmapLevels::None).unwrap();
        let framebuffer = Framebuffer::new(ctx, &[
            (FramebufferAttachment::Color(0), &framebuffer_dst)
        ]).unwrap();

        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let resources = Rc::new(TileRenderResources::new(ctx).unwrap());

        let mut world = World::new(1337);
        world.scramble();
        let world = RenderableWorld::new(ctx, resources, world).unwrap();

        Self {
            pp_program,
            camera,
            world,
            framebuffer_dst,
            framebuffer,
            time: 0.0,
            screen_size
        }
    }

    fn draw(&mut self, ctx: &Context, delta: Duration) {
        self.time = self.time.add(delta.as_secs_f32() * 0.5).rem(10.0); //6.4;//
        ctx.clear(Rgba::new(23,23,23,255));
        self.world.update(delta);

        ctx.use_framebuffer(&self.framebuffer);
        self.world.render(ctx, &self.camera);

        ctx.use_framebuffer(None);
        ctx.set_blend_state(None);
        ctx.use_program(&self.pp_program);
        ctx.set_uniform(&self.pp_program.get_uniform("time").unwrap(), self.time); //
        ctx.set_uniform(&self.pp_program.get_uniform("inv_camera").unwrap(), self.camera.to_matrix().inverse());
        ctx.set_uniform(&self.pp_program.get_uniform("pxRange").unwrap(), self.screen_size.height as f32 / (2.0 * self.camera.scale));
        ctx.bind_texture(0, &self.framebuffer_dst);
        ctx.draw_arrays(PrimitiveType::TriangleStrip, 0, 4);
    }

    fn event(&mut self, ctx: &Context, event: app::Event) {
        match event {
            Event::WindowResize(width, height) => if width > 0.0 && height > 0.0 {
                self.camera.aspect = width / height;
                self.framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(width as u32, height as u32),
                                                    InternalFormat::R8, MipmapLevels::None).unwrap();
                self.framebuffer.update_attachments(&[(FramebufferAttachment::Color(0), &self.framebuffer_dst)]).unwrap();
                self.screen_size = PhysicalSize::new(width as u32, height as u32);
            }
            Event::Click(pos) => {
                let pt = self.camera.to_world_coords(pos).into();
                self.world.try_rotate(pt);
                if self.world.is_completed() {
                    let mut new_world = World::new(self.world.seed() + 1);
                    new_world.scramble();
                    self.world.reinitialize(new_world);
                }

            },
            Event::Zoom(center, amount) => {
                let camera = &mut self.camera;
                let old = camera.to_world_coords(center);
                camera.scale = camera.scale.sub(amount * (camera.scale / 10.0)).max(1.0);
                let new = camera.to_world_coords(center);
                camera.position += old - new;
            }
            Event::Drag(delta) => {
                self.camera.position += self.camera.to_world_coords(-delta.absolute()) - self.camera.to_world_coords(Vec2::ZERO);
            },
            Event::DragEnd(_) => {},
            Event::DragStart(_) => {}
        }
    }
}

