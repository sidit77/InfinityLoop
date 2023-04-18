use jni::{objects::JObject, sys::jobject};

pub fn enable_immersive() -> infinity_loop::export::Result<()> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let ctx = unsafe { JObject::from_raw(ctx.context() as jobject)};
    let window = env
        .call_method(&ctx, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    let view = env
        .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
    let window_insets_controller = env
        .call_method(view, "getWindowInsetsController", "()Landroid/view/WindowInsetsController;", &[])?.l()?;

    if window_insets_controller.is_null() {
        return Ok(());
    }

    let window_insets_controller_class = env
        .find_class("android/view/WindowInsetsController")?;
    let flag_behavior_show_bars_by_swipe = env
        .get_static_field(window_insets_controller_class, "BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE", "I")?;
    env.call_method(&window_insets_controller, "setSystemBarsBehavior", "(I)V", &[flag_behavior_show_bars_by_swipe.borrow()])?;
    let window_insets_type_class = env.find_class("android/view/WindowInsets$Type")?;
    let system_bars = env.call_static_method(window_insets_type_class, "systemBars", "()I", &[])?;
    env.call_method(&window_insets_controller, "hide", "(I)V", &[system_bars.borrow()])?;

    let layout_params = env.find_class("android/view/WindowManager$LayoutParams")?;
    let cutout_mode = env.get_static_field(layout_params, "LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES", "I")?;

    let window_attributes = env
        .call_method(&window, "getAttributes", "()Landroid/view/WindowManager$LayoutParams;", &[])?
        .l()?;

    env.set_field(window_attributes, "layoutInDisplayCutoutMode", "I", cutout_mode.borrow())?;

    env.exception_clear()?;

    Ok(())
}