//Raytracing_Rust_Project/src/main.rs

use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;
// use rand::{Rng};

mod framebuffer;
mod ray_intersect;
mod sphere; 
mod color;
mod camera;
mod light;
mod material;
mod texture;

use framebuffer::Framebuffer;
use sphere::Sphere;
use color::Color;
use ray_intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use material::Material;

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        // Ray is entering the object
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        // Ray is leaving the object
        n_cosi = cosi;
        eta = eta_t;  // Assuming it's going back into air with index 1.0
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        // Total internal reflection
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Sphere],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Sphere],
    light: &Light,
    depth: u32, // this value should initially be 0
                // and should be increased by 1 in each recursion
) -> Color {
    if depth > 3 {  // default recursion depth
        return SKYBOX_COLOR; // Max recursion depth reached
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        // return default sky box color
        return SKYBOX_COLOR;
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse_color = intersect.material.get_diffuse_color(intersect.u, intersect.v);
    let diffuse = diffuse_color * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }


    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }

    (diffuse + specular) * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    // random number generator
    // let mut rng = rand::thread_rng();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // if rng.gen_range(0.0..1.0) < 0.9 {
            //      continue;
            // }

            // Map the pixel coordinate to screen space [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio and perspective 
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Apply camera rotation to the ray direction
            let rotated_direction = camera.basis_change(&ray_direction);

            // Cast the ray and get the pixel color
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Raytracer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    // let rubber = Material::new(
    //     Color::new(255, 100, 80),
    //     1.0,
    //     [0.9, 0.1, 0.0, 0.0],
    //     0.0,
    // );

    let rubber = Material::new_with_texture(
        1.0,
        [0.9, 0.1, 0.0, 0.0],
        0.0,
    );

    let ivory = Material::new(
        Color::new(100, 100, 80),
        50.0,
        [0.6, 0.3, 0.6, 0.0],
        0.0,
    );

    let glass = Material::new(
        Color::new(255, 255, 255),
        1425.0,
        [0.0, 10.0, 0.5, 0.5],
        0.3,
    );

    let objects = [
        Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 1.0, material: rubber },
        Sphere { center: Vec3::new(-1.0, -1.0, 1.5), radius: 0.5, material: ivory },
        Sphere { center: Vec3::new(-0.3, 0.3, 1.5), radius: 0.3, material: glass },
        // Sphere { center: Vec3::new(-2.0, 2.0, -5.0), radius: 1.0, material: ivory },
    ];

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),  // eye: Initial camera position
        Vec3::new(0.0, 0.0, 0.0),  // center: Point the camera is looking at (origin)
        Vec3::new(0.0, 1.0, 0.0)   // up: World up vector
    );
    let rotation_speed = PI/50.0;
    let zoom_speed = 0.1;

    let light = Light::new(
        Vec3::new(1.0, -1.0, 5.0),
        Color::new(255, 255, 255),
        1.0
    );

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        // camera zoom controls
        if window.is_key_down(Key::Q) {
            camera.zoom(zoom_speed);
        }
        if window.is_key_down(Key::E) {
            camera.zoom(-zoom_speed);
        }

        if camera.is_changed() {
            // Render the scene
            render(&mut framebuffer, &objects, &camera, &light);
        }

        // update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}