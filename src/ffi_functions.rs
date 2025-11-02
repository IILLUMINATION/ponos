use super::{
    MoonWalkState,
    ffi_utils::{mat4_from_ptr, string_from_ptr},
    font::FontId,
    glam::{Vec2, Vec3, Vec4},
    objects::UniformValue,
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, WindowHandle, DisplayHandle};

#[derive(Clone, Copy)]
struct WindowHandleWrapper {
    window_handle: raw_window_handle::RawWindowHandle,
    display_handle: raw_window_handle::RawDisplayHandle,
}

impl HasWindowHandle for WindowHandleWrapper {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe { WindowHandle::borrow_raw(self.window_handle) })
    }
}

impl HasDisplayHandle for WindowHandleWrapper {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(self.display_handle) })
    }
}

unsafe impl Send for WindowHandleWrapper {}
unsafe impl Sync for WindowHandleWrapper {}


#[no_mangle]
pub unsafe extern "C" fn moonwalk_init(window_handle: raw_window_handle::RawWindowHandle, display_handle: raw_window_handle::RawDisplayHandle) -> *mut MoonWalkState<'static> {
    let handle_wrapper = Box::new(WindowHandleWrapper {
        window_handle,
        display_handle,
    });
    
    let static_handle_wrapper: &'static WindowHandleWrapper = Box::leak(handle_wrapper);

    match MoonWalkState::new(static_handle_wrapper) {
        Ok(state) => Box::into_raw(Box::new(state)),
        Err(e) => {
            eprintln!("MoonWalk initialization failed: {}", e);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_shutdown(state_ptr: *mut MoonWalkState) {
    if !state_ptr.is_null() {
        drop(Box::from_raw(state_ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_set_viewport(state_ptr: *mut MoonWalkState, width: u32, height: u32) {
    if let Some(state) = state_ptr.as_mut() {
        state.set_viewport(width, height);
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_render_frame(state_ptr: *mut MoonWalkState, r: f32, g: f32, b: f32, a: f32) {
    if let Some(state) = state_ptr.as_mut() {
        if let Err(e) = state.render_frame(Vec4::new(r, g, b, a)) {
            eprintln!("MoonWalk render failed: {}", e);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_new_rect(state_ptr: *mut MoonWalkState) -> u32 {
    state_ptr.as_mut().map_or(0, |s| s.new_rect().to_u32())
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_new_text(state_ptr: *mut MoonWalkState) -> u32 {
    state_ptr.as_mut().map_or(0, |s| s.new_text().to_u32())
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_position(state_ptr: *mut MoonWalkState, id: u32, x: f32, y: f32) {
    if let Some(state) = state_ptr.as_mut() {
        state.config_position(id.into(), Vec2::new(x, y));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_size(state_ptr: *mut MoonWalkState, id: u32, width: f32, height: f32) {
    if let Some(state) = state_ptr.as_mut() {
        state.config_size(id.into(), Vec2::new(width, height));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_rotation(state_ptr: *mut MoonWalkState, id: u32, angle_degrees: f32) {
    if let Some(state) = state_ptr.as_mut() {
        state.config_rotation(id.into(), angle_degrees);
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_color(state_ptr: *mut MoonWalkState, id: u32, r: f32, g: f32, b: f32, a: f32) {
    if let Some(state) = state_ptr.as_mut() {
        state.config_color(id.into(), Vec4::new(r, g, b, a));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_z_index(state_ptr: *mut MoonWalkState, id: u32, z: f32) {
    if let Some(state) = state_ptr.as_mut() {
        state.config_z_index(id.into(), z);
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_text(state_ptr: *mut MoonWalkState, id: u32, text_ptr: *const libc::c_char) {
    if let Some(state) = state_ptr.as_mut() {
        if let Ok(text) = string_from_ptr(text_ptr) {
            state.config_text(id.into(), &text);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_load_font(state_ptr: *mut MoonWalkState, path_ptr: *const libc::c_char, size: f32) -> u64 {
    let state = if let Some(s) = state_ptr.as_mut() { s } else { return 0; };
    let path = if let Ok(p) = string_from_ptr(path_ptr) { p } else { return 0; };

    match state.load_font(&path, size) {
        Ok(font_id) => font_id.to_u64(),
        Err(e) => {
            eprintln!("Failed to load font from {}: {}", path, e);
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_clear_font(state_ptr: *mut MoonWalkState, font_id: u64) {
    if let Some(state) = state_ptr.as_mut() {
        state.clear_font(FontId::from_u64(font_id));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_config_font(state_ptr: *mut MoonWalkState, object_id: u32, font_id: u64) {
    if let Some(state) = state_ptr.as_mut() {
        state.config_font(object_id.into(), FontId::from_u64(font_id));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_set_rounded(state_ptr: *mut MoonWalkState, object_id: u32, tl: f32, tr: f32, br: f32, bl: f32) {
    if let Some(state) = state_ptr.as_mut() {
        state.set_rounded(object_id.into(), Vec4::new(tl, tr, br, bl));
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_delete_object(state_ptr: *mut MoonWalkState, id: u32) {
    if let Some(state) = state_ptr.as_mut() {
        state.delete_object(id.into());
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_clear_all(state_ptr: *mut MoonWalkState) {
    if let Some(state) = state_ptr.as_mut() {
        state.clear_all();
    }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_compile_shader(state_ptr: *mut MoonWalkState, vs_src_ptr: *const libc::c_char, _fs_src_ptr: *const libc::c_char) -> u32 {
    if let Some(state) = state_ptr.as_mut() {
        if let Ok(shader_src) = string_from_ptr(vs_src_ptr) {
            match state.compile_shader(&shader_src) {
                Ok(id) => id.to_u32(),
                Err(e) => {
                    eprintln!("Shader compilation failed: {}", e);
                    0
                }
            }
        } else { 0 }
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn moonwalk_set_object_shader(state_ptr: *mut MoonWalkState, object_id: u32, shader_id: u32) {
    if let Some(state) = state_ptr.as_mut() {
        state.set_object_shader(object_id.into(), shader_id.into());
    }
}

macro_rules! define_set_uniform {
    ($func_name:ident, $type:ty, $value_expr:expr, $($param:ident: $ptype:ty),*) => {
        #[no_mangle]
        pub unsafe extern "C" fn $func_name(state_ptr: *mut MoonWalkState, id: u32, name: *const libc::c_char, $($param: $ptype),*) {
            if let Some(s) = state_ptr.as_mut() {
                if let Ok(n) = string_from_ptr(name) {
                    s.set_uniform(id.into(), n, $value_expr);
                }
            }
        }
    };
}

define_set_uniform!(moonwalk_set_uniform_int, UniformValue, UniformValue::Int(val), val: i32);
define_set_uniform!(moonwalk_set_uniform_float, UniformValue, UniformValue::Float(val), val: f32);
define_set_uniform!(moonwalk_set_uniform_vec2, UniformValue, UniformValue::Vec2(Vec2::new(x, y)), x: f32, y: f32);
define_set_uniform!(moonwalk_set_uniform_vec3, UniformValue, UniformValue::Vec3(Vec3::new(x, y, z)), x: f32, y: f32, z: f32);
define_set_uniform!(moonwalk_set_uniform_vec4, UniformValue, UniformValue::Vec4(Vec4::new(x, y, z, w)), x: f32, y: f32, z: f32, w: f32);
define_set_uniform!(moonwalk_set_uniform_bool, UniformValue, UniformValue::Bool(val), val: bool);

#[no_mangle]
pub unsafe extern "C" fn moonwalk_set_uniform_mat4(state_ptr: *mut MoonWalkState, id: u32, name: *const libc::c_char, mat_ptr: *const f32) {
    if let Some(s) = state_ptr.as_mut() {
        if let (Ok(n), Ok(m)) = (string_from_ptr(name), mat4_from_ptr(mat_ptr)) {
            s.set_uniform(id.into(), n, UniformValue::Mat4(m));
        }
    }
}