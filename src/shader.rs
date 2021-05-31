use web_sys::{WebGl2RenderingContext, WebGlProgram};

pub fn compile_program(context: &WebGl2RenderingContext, shaders: &[(u32, &str)]) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    for &(shader_type, source) in shaders {
        let shader = context
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        context.shader_source(&shader, source);
        context.compile_shader(&shader);

        if context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            context.attach_shader(&program, &shader);
        } else {
            return Err(context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }

}

