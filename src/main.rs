/// Imports
extern crate sdl2;
extern crate png;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use std::f32;
use std::cmp;

/// Types
#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
}
type Vector = Point;
#[derive(Debug)]
struct State {
    position: Point,
    samples: u32,
    direction: f32,
    fov: f32,
    fog: bool,
}

/// Constants
const WIDTH: u32 = 800;
const HEIGHT: u32 = 480;
const SPEED: f32 = 0.5;
// 1 deg * 2
const ROT_SPEED: f32 = 0.017453292519943 * 2.0;
const STARTING_POSITION: Point = Point{ x: 0.0, y: 0.0 };
// 75 deg
const FOV: f32 = f32::consts::PI * 0.416;
const SAMPLES: u32 = WIDTH / 10;
const STARTING_DIRECTION: f32 = f32::consts::PI / 4.0;

/// Helper Functions
// TOOD: turn these into macros
fn rad(deg: f32) -> f32 {
    return (deg / 180.0) * f32::consts::PI;
}
fn fmin(a: f32, b:f32) -> f32 {
    if a < b {
        return a;
    } else {
        return b;
    }
}
fn fmax(a: f32, b:f32) -> f32 {
    if a < b {
        return b;
    } else {
        return a;
    }
}
fn angle_to_vec(theta: f32) -> Vector {
    return Vector { x: f32::cos(theta), y: f32::sin(theta) };
}
// Intersection algorithm AABB
// Returns distance of intersection if it exists.
fn intersect(origin: Point, vec: Vector, cube: Point) -> Option<f32> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    if vec.x != 0.0 {
        let tx1 = (cube.x - origin.x) / vec.x;
        let tx2 = (cube.x + 1.0 - origin.x) / vec.x;
        tmin = fmax(tmin, fmin(tx1, tx2));
        tmax = fmin(tmax, fmax(tx1, tx2));
    }
    if vec.y != 0.0 {
        let ty1 = (cube.y - origin.y) / vec.y;
        let ty2 = (cube.y + 1.0 - origin.y) / vec.y;
        tmin = fmax(tmin, fmin(ty1, ty2));
        tmax = fmin(tmax, fmax(ty1, ty2));
    }
    if tmax >= tmin && tmin >= 0.0 {
        let dist = f32::sqrt(f32::powf(vec.x * tmin, 2.0) + f32::powf(vec.y * tmin, 2.0));
        Some(dist)
    } else {
        None
    }
}
fn gen_map() -> Vec<Point> {
    let mut map = Vec::new();
    map.push(Point{ x: 1.0, y: 1.0 });
    map.push(Point{ x: 3.0, y: 3.0 });
    map.push(Point{ x: 3.0, y: 4.0 });
    map.push(Point{ x: 1.0, y: 6.0 });
    map.push(Point{ x: 3.0, y: 5.0 });
    map.push(Point{ x: 5.0, y: 3.0 });
    map.push(Point{ x: 5.0, y: 4.0 });
    map.push(Point{ x: 5.0, y: 5.0 });
    map
}
// TODO: Why does this still cause minor distortion?
fn distance_to_height(dist: f32, angle: f32) -> f32 {
    return ((HEIGHT) as f32) / (dist * f32::cos(angle));
}

fn draw_rect(canvas: &mut Canvas<Window>, state: &State, x: u32, height: f32, width: u32) {
    if state.fog {
        let alpha = ((1.0 - fmin(1.0, ((height + HEIGHT as f32 / 2.0) / (HEIGHT as f32)))) * 255.0) as u8;
        canvas.set_draw_color(Color::RGB(alpha, alpha, alpha));
    } else {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
    }
    let height = cmp::min(height as u32, HEIGHT);
    let x = x as i32;
    let y = ((HEIGHT / 2) - (height / 2)) as i32;
    canvas.fill_rect(Rect::new(x, y, width, height));
}

fn create_rect(x: u32, rect_width: u32, rect_height: u32, image_height: u32) -> (Rect, Color) {
    let alpha = ((1.0 - fmin(1.0, (((rect_height + rect_height) as f32 / 2.0) / (rect_height as f32)))) * 255.0) as u8;
    let x = x as i32;
    let y = ((image_height / 2) - (rect_height / 2)) as i32;
    (Rect::new(x, y, rect_width, rect_height), Color::RGB(alpha, alpha, alpha))
}

struct ProceduralGenerator {
    start_time: Instant,
    map: Vec<Point>,
}

// pixel width in bytes
const IMAGE_PIXEL_WIDTH: usize = 3;
// Image is just a buffer with the associated metadata
struct Image {
    buf: Vec<u8>,
    pixel_width: usize,
    pixel_height: usize,
}

impl Image {
    // TODO: remove SDL2 dependency
    pub fn draw_rect(
        &mut self,
        rect: Rect,
        color: Color,
    ) {
        let (x, y, width, height) = (rect.x() as usize, rect.y() as usize, rect.width() as usize, rect.height() as usize);
        for i in 0..width {
            for j in 0..height {
                let xi = x + i;
                let yi = y + j;
                self.draw_pixel(xi, yi, color);
            }
        }

    }

    fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = ((y * self.pixel_width) + x) * IMAGE_PIXEL_WIDTH;
        self.buf[index] = color.r;
        self.buf[index + 1] = color.g;
        self.buf[index + 2] = color.b;
    }
}

impl ProceduralGenerator {
    pub fn new() -> ProceduralGenerator {
        ProceduralGenerator {
            start_time: Instant::now(),
            // TODO: generate the map
            map: gen_map(),
        }
    }

    // TODO: use timestamp w/ start_time
    fn get_image(&self, width: u32, height: u32, timestamp: Option<Instant>) -> Image {
        // Render pass: create rectangles
        let mut theta = STARTING_DIRECTION + (FOV / 2.0);
        let delta_theta = FOV / (SAMPLES as f32);
        let rect_width = width / SAMPLES;
        let mut rects = Vec::new();
        for i in 0..SAMPLES {
            let vector = angle_to_vec(theta);
            let mut dist = f32::INFINITY;
            for cube in &self.map {
                if let Some(intersection_distance) = intersect(STARTING_POSITION, vector, *cube) {
                    dist = fmin(dist, intersection_distance);
                }
            }
            if dist < f32::INFINITY {
                let rect_height = cmp::min(
                    distance_to_height(dist, (STARTING_DIRECTION - theta).abs()) as u32,
                    height,
                );
                rects.push(create_rect(i * rect_width, rect_width, rect_height, height))
            }
            theta -= delta_theta;
        }

        // Create and write to buffer
        let (w, h) = (width as usize, height as usize);
        let buffer_size = w * h * IMAGE_PIXEL_WIDTH;
        let mut image = Image {
            buf: vec![255; buffer_size],
            pixel_width: width as usize,
            pixel_height: height as usize,
        };
        for (rect, color) in rects {
            image.draw_rect(rect, color);
        }
        image
    }
}

fn render(canvas: &mut Canvas<Window>,
          map: &Vec<Point>,
          state: &State) {
    let State { position, samples, direction, fov, fog } = *state;
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut theta = direction + (fov / 2.0);
    let delta_theta = fov / (samples as f32);
    let width = WIDTH / samples;
    for i in 0..samples {
        let vector = angle_to_vec(theta);
        let mut dist = f32::INFINITY;
        for cube in map {
            if let Some(intersection_distance) = intersect(position, vector, *cube) {
                dist = fmin(dist, intersection_distance);
            }
        }
        if dist < f32::INFINITY {
            let height = distance_to_height(dist, (direction - theta).abs());
            draw_rect(canvas, state, i * width, height, width);
        }
        theta -= delta_theta;
    }
    canvas.present();
}

fn write_png(image: Image) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let path_string = format!("{}.png", timestamp.as_secs().to_string());
    println!("Saving with filename {}", path_string);
    let path = Path::new(&path_string);
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image.pixel_width as u32, image.pixel_height as u32);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image.buf).expect("Failed to write image buffer to png");
}

/// Main
fn main() {
    // Canvas setup
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("procedural-raytracer", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    // Scene setup
    let mut render_flag = true;
    let mut state = State {
      position: Point { x: 0.0, y: 0.0 },
      fov: f32::consts::PI * 0.416,
      samples: 800,
      direction: f32::consts::PI / 4.0,
      fog: true,
    };
    let map = gen_map();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let procedural_generator = ProceduralGenerator::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                  Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
                  Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    let vector = angle_to_vec(state.direction);
                    state.position.x += SPEED * vector.x;
                    state.position.y += SPEED * vector.y;
                    println!("Current position: ({}, {})", state.position.x, state.position.y);
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    state.direction += ROT_SPEED;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    let vector = angle_to_vec(state.direction);
                    state.position.x -= SPEED * vector.x;
                    state.position.y -= SPEED * vector.y;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    state.direction -= ROT_SPEED;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    // Render an image and dump it locally
                    let image = procedural_generator.get_image(WIDTH, HEIGHT, None);
                    write_png(image);
                },
                // Decrease fov
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    state.fov -= rad(1.0);
                    render_flag = true;
                },
                // Increase fov
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    state.fov += rad(1.0);
                    render_flag = true;
                },
                // Decrease Resolution
                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
                    state.samples /= 2;
                    state.samples = cmp::max(1, state.samples);
                    render_flag = true;
                },
                // Increase Resolution
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
                    state.samples *= 2;
                    state.samples = cmp::min(state.samples, WIDTH);
                    render_flag = true;
                },
                // Toggle fog mode
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    state.fog = !state.fog;
                    render_flag = true;
                },
                _ => {}
            }
        }
        if render_flag {
            render(&mut canvas, &map, &state);
            render_flag = false;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
