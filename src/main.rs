use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use crate::color::Color;
use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{earth_shader,  jupiter_shader, mars_shader, 
    moon_shader, sun_shader, vertex_shader, comet_shader, saturn_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType};

pub struct Uniforms<'a> {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: &'a FastNoiseLite,  // Pasamos referencia
}


// Reutilizamos la instancia de ruido para evitar recrearla en cada frame
fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

// Resto de funciones de matrices (sin cambios)
fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    // Rotación y transformación (sin cambios)
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, cos_x, -sin_x, 0.0,
        0.0, sin_x, cos_x, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sin_y, 0.0, cos_y, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z, cos_z, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;
    let transform_matrix = Mat4::new(
        scale, 0.0, 0.0, translation.x,
        0.0, scale, 0.0, translation.y,
        0.0, 0.0, scale, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    transform_matrix * rotation_matrix
}

// Funciones de vista y proyección (sin cambios)
fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    perspective(fov, aspect_ratio, 0.1, 1000.0)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

// Render con post-procesamiento incluido
fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    shader_index: usize,
) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
    
        if x < framebuffer.width && y < framebuffer.height {
            let (color, emission) = match shader_index {
                0 => {
                    let color = sun_shader(uniforms);
                    (color, color.to_hex())  // Sol emisivo
                }
                1 => (earth_shader(&fragment, uniforms), 0),
                2 => (mars_shader(&fragment, uniforms), 0),
                3 => (jupiter_shader(&fragment, uniforms), 0),
                4 => (moon_shader(&fragment, uniforms), 0),
                5 => (saturn_shader(&fragment, uniforms), 0),
                6 => (comet_shader(&fragment, uniforms), 0),
                _ => (Color::black(), 0),
            };
    
            framebuffer.set_current_color(color.to_hex());
    
            // Solo escribimos en el buffer de emisión si hay emisión
            if emission != 0 {
                framebuffer.point_with_emission(x, y, fragment.depth, emission);
            } else {
                framebuffer.point_with_emission(x, y, fragment.depth, 0);
            }
        }
    }
    
}
fn post_process(framebuffer: &mut Framebuffer) {
    for (pixel, emission) in framebuffer.buffer.iter_mut().zip(&framebuffer.emission_buffer) {
        if *emission != 0 {
            *pixel = blend_emission(*pixel, *emission);
        }
    }
}

// Nueva función de mezcla usando interpolación
fn blend_emission(color: u32, emission: u32) -> u32 {
    let r1 = (color >> 16) & 0xFF;
    let g1 = (color >> 8) & 0xFF;
    let b1 = color & 0xFF;

    let r2 = (emission >> 16) & 0xFF;
    let g2 = (emission >> 8) & 0xFF;
    let b2 = emission & 0xFF;

    // Interpolación suave entre los dos colores (lerp)
    let blend = |c1, c2| ((c1 as f32 * 0.8) + (c2 as f32 * 0.2)).min(255.0) as u32;

    let r = blend(r1, r2);
    let g = blend(g1, g2);
    let b = blend(b1, b2);

    (r << 16) | (g << 8) | b
}

fn main() {
    let window_width = 800;
    let window_height = 800;
    let framebuffer_width = 800;
    let framebuffer_height = 800;  // Ajustamos a 800x800 para evitar deformaciones

    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Animated Fragment Shader",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let scale = 1.0;

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let obj = Obj::load("assets/models/sphere-1.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();

    let mut time = 0;
    let noise = create_noise();
    let mut shader_index = 0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;
        handle_input(&window, &mut camera, &mut shader_index);

        framebuffer.clear();

        let model_matrix = create_model_matrix(translation, scale, rotation);
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(
            window_width as f32,
            window_height as f32,
        );
        let viewport_matrix = create_viewport_matrix(
            framebuffer_width as f32,
            framebuffer_height as f32,
        );

        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            noise: &noise,  // ¡Solución aquí! No intentamos clonar.
        };

        render(&mut framebuffer, &uniforms, &vertex_arrays, shader_index);

        // Aplicamos post-procesamiento después del renderizado
        post_process(&mut framebuffer);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

// Manejo de entrada para controlar la cámara y cambiar shaders
fn handle_input(window: &Window, camera: &mut Camera, shader_index: &mut usize) {
    if window.is_key_down(Key::Key1) { *shader_index = 0; }
    if window.is_key_down(Key::Key2) { *shader_index = 1; }
    if window.is_key_down(Key::Key3) { *shader_index = 2; }
    if window.is_key_down(Key::Key4) { *shader_index = 3; }
    if window.is_key_down(Key::Key5) { *shader_index = 4; }
    if window.is_key_down(Key::Key6) { *shader_index = 5; }
    if window.is_key_down(Key::Key7) { *shader_index = 6; }

    let movement_speed = 1.0;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.1;

    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }

    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }
}
