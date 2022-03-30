use glow::HasContext;
use crate::{Context, InternalFormat};
use crate::opengl::{GlError, GlResult, Texture};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FramebufferAttachment {
    Color(u32),
    Depth,
    Stencil,
    DepthStencil
}

impl FramebufferAttachment {
    pub fn raw(self) -> u32 {
        match self {
            FramebufferAttachment::Color(i) => glow::COLOR_ATTACHMENT0 + i,
            FramebufferAttachment::Depth => glow::DEPTH_ATTACHMENT,
            FramebufferAttachment::Stencil => glow::STENCIL_ATTACHMENT,
            FramebufferAttachment::DepthStencil => glow::DEPTH_STENCIL_ATTACHMENT
        }
    }
}


type GlowFramebuffer = glow::Framebuffer;

pub struct Framebuffer {
    ctx: Context,
    id: GlowFramebuffer
}

impl Framebuffer {

    pub fn new<'a>(ctx: &Context, attachments: &[(FramebufferAttachment, &'a dyn FramebufferDestination)]) -> GlResult<Self> {
        let gl = ctx.raw();
        let framebuffer = unsafe {
            let id = gl.create_framebuffer()?;
            Self {
                ctx: ctx.clone(),
                id
            }
        };
        framebuffer.update_attachments(attachments)?;
        Ok(framebuffer)
    }

    pub fn update_attachments<'a>(&self, attachments: &[(FramebufferAttachment, &'a dyn FramebufferDestination)]) -> GlResult<()>{
        self.ctx.use_framebuffer(Some(self));
        for (attachment, destination) in attachments {
            destination.attach(self, *attachment);
        }
        verify_framebuffer_status(self.ctx.raw())
    }

    pub fn raw(&self) -> GlowFramebuffer {
        self.id
    }

}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let gl = self.ctx.raw();
        unsafe {
            gl.delete_framebuffer(self.id);
        }
    }
}

pub trait FramebufferDestination {
    fn attach(&self, framebuffer: &Framebuffer, attachment: FramebufferAttachment);
}


impl FramebufferDestination for Texture {
    fn attach(&self, framebuffer: &Framebuffer, attachment: FramebufferAttachment) {
        unsafe {
            framebuffer.ctx.raw().framebuffer_texture_2d(glow::FRAMEBUFFER, attachment.raw(), glow::TEXTURE_2D, Some(self.raw()), 0)
        }
    }
}


type GlowRenderbuffer = glow::Renderbuffer;

pub struct Renderbuffer {
    ctx: Context,
    id: GlowRenderbuffer
}

impl Renderbuffer {

    pub fn new(ctx: &Context, format: InternalFormat, width: u32, height: u32) -> GlResult<Self> {
        let gl = ctx.raw();
        let rb = unsafe {
            let id = gl.create_renderbuffer()?;
            Self {
                ctx: ctx.clone(),
                id
            }
        };
        rb.resize(format, width, height);
        Ok(rb)
    }

    pub fn resize(&self, format: InternalFormat, width: u32, height: u32) {
        self.ctx.bind_renderbuffer(self);
        let gl = self.ctx.raw();
        unsafe {
            gl.renderbuffer_storage(glow::RENDERBUFFER, format.raw(), width as i32, height as i32);
        }
    }

    pub fn raw(&self) -> GlowRenderbuffer {
        self.id
    }

}

impl Drop for Renderbuffer {
    fn drop(&mut self) {
        let gl = self.ctx.raw();
        unsafe {
            gl.delete_renderbuffer(self.id);
        }
    }
}

impl FramebufferDestination for Renderbuffer {
    fn attach(&self, _: &Framebuffer, attachment: FramebufferAttachment) {
        unsafe {
            self.ctx.raw().framebuffer_renderbuffer(glow::FRAMEBUFFER, attachment.raw(), glow::RENDERBUFFER, Some(self.raw()));
        }
    }
}

fn verify_framebuffer_status(gl: &glow::Context) -> GlResult<()> {
    unsafe {
        match gl.check_framebuffer_status(glow::FRAMEBUFFER) {
            glow::FRAMEBUFFER_COMPLETE => Ok(()),
            glow::FRAMEBUFFER_UNDEFINED =>
                Err(GlError::from("The specified framebuffer is the default read or draw framebuffer, but the default framebuffer does not exist")),
            glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT =>
                Err(GlError::from("One of the framebuffer attachment points is framebuffer incomplete")),
            glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT =>
                Err(GlError::from("The framebuffer does not have at least one image attached to it")),
            glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER =>
                Err(GlError::from("The value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for any color attachment point(s) named by GL_DRAW_BUFFERi")),
            glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER =>
                Err(GlError::from("GL_READ_BUFFER is not GL_NONE and the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is \
                        GL_NONE for the color attachment point named by GL_READ_BUFFER")),
            glow::FRAMEBUFFER_UNSUPPORTED =>
                Err(GlError::from("The combination of internal formats of the attached images violates an implementation-dependent set of restrictions")),
            glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE =>
                Err(GlError::from("The value of GL_RENDERBUFFER_SAMPLES is not the same for all attached renderbuffers, \
                        the value of GL_TEXTURE_SAMPLES is the not same for all attached textures, \
                        the attached images are a mix of renderbuffers and textures, \
                        the value of GL_RENDERBUFFER_SAMPLES does not match the value of GL_TEXTURE_SAMPLES, \
                        the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not the same for all attached textures, \
                        the attached images are a mix of renderbuffers and textures, or \
                        the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not GL_TRUE for all attached textures.")),
            glow::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS =>
                Err(GlError::from("Any framebuffer attachment is layered, and any populated attachment is not layered, \
                        or all populated color attachments are not from textures of the same target")),
            _ => Err(GlError::from("An error occurred!"))
        }
    }
}