pub mod audio;
pub mod debug;
pub mod input;
pub mod render;
pub mod ui;
pub mod vfx;

pub fn register_placeholders() {
    render::register();
    input::register();
    ui::register();
    audio::register();
    vfx::register();
    debug::register();
}
