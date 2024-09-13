use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Duration;
use nalgebra_glm::{Vec2, distance};
use std::f32::consts::PI;

mod framebuffer;
use framebuffer::Framebuffer;


fn render(framebuffer: &mut Framebuffer) {

}


fn main() {
    let window_width = 800;
    let window_height = 600;

    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let frame_delay = Duration::from_millis(0);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    
    let mut window = Window::new(
        "Rust Graphics - Raytracer",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();
    
    window.set_position(100, 100);
    window.update();

    framebuffer.set_background_color(0x000000);

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }
        framebuffer.clear();
        render(&mut framebuffer);
        
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
        std::thread::sleep(frame_delay);
    }
}