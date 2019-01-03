/// Imports
extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;
use std::f32;
use std::cmp;

/// Types
struct Point {
    x: f32,
    y: f32,
}
type Vector = Point;

/// Constants
// 75 deg
const FOV: f32 = f32::consts::PI * 0.416;
const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const SPEED: f32 = 0.1;
// 1 deg * 2
const ROT_SPEED: f32 = 0.017453292519943 * 2.0;

/// Helper Functions
// TOOD: turn these into macros
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
// - Returns -1 on failed intersection, otherwise returns distance
// TODO: Consider changing it -> an optional on failed intersection (it'll clean up other areas)
fn intersect(origin: &Point, vec: &Vector, cube: &Point) -> f32 {
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
    if tmax >= tmin && tmax >= 0.0 {
        let dist = f32::sqrt(f32::powf(vec.x * tmin, 2.0) + f32::powf(vec.y * tmin, 2.0));
        return dist;
    } else {
        // TODO: Change this to a constant
        return -1.0;
    }
}
// TODO: Figure out wtf this does, I basically copied from Python code
fn distance_to_height(dist: f32, angle: f32) -> f32 {
    return ((HEIGHT) as f32 - 50.0) / (dist * f32::cos(angle));
}
fn draw_rect(canvas: &mut Canvas<Window>, x: u32, height: f32, width: u32) {
    let alpha = ((1.0 - (height / (HEIGHT as f32))) * 255.0) as u8;
    canvas.set_draw_color(Color::RGB(alpha, alpha, alpha));
    let height = cmp::min(height as u32, HEIGHT);
    let x = x as i32;
    let y = ((HEIGHT / 2) - (height / 2)) as i32;
    //println!("x: {}, y: {}, width: {}, height: {}", x, y, width, height);
    canvas.fill_rect(Rect::new(x, y, width, height));
}
fn gen_map(map: &mut Vec<Point>) {
    map.push(Point{ x: 1.0, y: 1.0 });
    map.push(Point{ x: 3.0, y: 3.0 });
    map.push(Point{ x: 3.0, y: 4.0 });
    map.push(Point{ x: 1.0, y: 6.0 });
    map.push(Point{ x: 3.0, y: 5.0 });
    map.push(Point{ x: 5.0, y: 3.0 });
    map.push(Point{ x: 5.0, y: 4.0 });
    map.push(Point{ x: 5.0, y: 5.0 });
}
fn render(canvas: &mut Canvas<Window>,
          position: &Point,
          map: &Vec<Point>,
          samples: u32,
          direction: f32) {
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut theta = direction + (FOV / 2.0);
    let delta_theta = FOV / (samples as f32);
    let width = WIDTH / samples;
    for i in 0..samples {
        let vector = angle_to_vec(theta);
        let mut dist = f32::NEG_INFINITY;
        for cube in map {
            let temp = intersect(&position, &vector, cube);
            if temp > 0.0 {
                if dist == f32::NEG_INFINITY {
                    dist = temp;
                } else {
                    dist = fmin(dist, temp);
                }
            }
        }
        if dist > 0.0 {
            let height = distance_to_height(dist, (direction - theta).abs());
            draw_rect(canvas, i * width, height, width);
        }
        theta -= delta_theta;
    }
    canvas.present();
}

/// Main
fn main() {
    // Canvas setup
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo: Video", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    // Scene setup
    let mut render_flag = true;
    let mut position = Point { x: 0.0, y: 0.0 };
    let mut map: Vec<Point> = Vec::new();
    gen_map(&mut map);
    let samples = 800;
    let mut direction = f32::consts::PI / 4.0;

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    let vector = angle_to_vec(direction);
                    position.x += SPEED * vector.x;
                    position.y += SPEED * vector.y;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    direction += ROT_SPEED;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    let vector = angle_to_vec(direction);
                    position.x -= SPEED * vector.x;
                    position.y -= SPEED * vector.y;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    direction -= ROT_SPEED;
                    render_flag = true;
                },
                _ => {}
            }
        }
        if render_flag {
            render(&mut canvas, &position, &map, samples, direction);
            render_flag = false;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
