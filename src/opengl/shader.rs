use glam::Mat4;
use glow::HasContext;
use crate::opengl::Context;
use crate::ShaderType;

type GlowProgram = glow::Program;
type GlowShader = glow::Shader;

pub type UniformLocation = glow::UniformLocation;

pub struct Shader {
    ctx: Context,
    id: GlowShader
}

impl Shader {

    pub fn new(ctx: &Context, shader_type: ShaderType, source: &str) -> Result<Self, String> {
        unsafe {
            let gl = ctx.raw();
            let id = gl.create_shader(shader_type.raw())?;
            gl.shader_source(id, source);
            gl.compile_shader(id);
            match gl.get_shader_compile_status(id) {
                true => Ok(Self {
                    ctx: ctx.clone(),
                    id
                }),
                false => Err(gl.get_shader_info_log(id))
            }
        }
    }

    pub fn raw(&self) -> GlowShader {
        self.id
    }

}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            let gl = self.ctx.raw();
            gl.delete_shader(self.id);
        }
    }
}

pub struct ShaderProgram {
    ctx: Context,
    id: GlowProgram
}

impl ShaderProgram {

    pub fn new(ctx: &Context, shaders: &[&Shader]) -> Result<Self, String> {
        unsafe {
            let gl = ctx.raw();
            let id = gl.create_program()?;
            for shader in shaders {
                gl.attach_shader(id, shader.raw());
            }
            gl.link_program(id);
            for shader in shaders {
                gl.detach_shader(id, shader.raw());
            }
            match gl.get_program_link_status(id) {
                true => Ok(Self {
                    ctx: ctx.clone(),
                    id
                }),
                false => Err(gl.get_program_info_log(id))
            }
        }
    }

    pub fn raw(&self) -> GlowProgram {
        self.id
    }

}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            let gl = self.ctx.raw();
            gl.delete_program(self.id);
        }
    }
}

pub trait GetUniformName {
    fn get_uniform_name(&self, name: &str) -> Option<UniformLocation>;
}

impl GetUniformName for ShaderProgram {
    fn get_uniform_name(&self, name: &str) -> Option<UniformLocation>  {
        let gl = self.ctx.raw();
        unsafe {
            gl.get_uniform_location(self.id, name)
        }
    }
}

pub trait SetUniform<T>: GetUniformName {
    fn set_uniform(&self, location: &UniformLocation, data: T);
    fn set_uniform_by_name(&self, name: &str, data: T) {
        self.set_uniform(&self.get_uniform_name(name).unwrap(), data)
    }
}

impl SetUniform<Mat4> for ShaderProgram {
    fn set_uniform(&self, location: &UniformLocation, data: Mat4) {
        unsafe {
            self.ctx.raw().uniform_matrix_4_f32_slice(Some(location), false, &data.to_cols_array())
        }
    }
}

