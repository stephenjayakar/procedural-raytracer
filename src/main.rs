/// Imports
use std::f32;

/// Types
struct Point {
    x: f32,
    y: f32,
}
type Vector = Point;

/// Constants
// PI / 3 = 60 deg
const FOV: f32 = f32::consts::PI / 3.0;
const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

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
fn angle_to_vec(theta: &f32) -> Vector {
    return Vector { x: f32::cos(*theta), y: f32::sin(*theta) };
}
// Intersection algorithm AABB
// - Returns -1 on failed intersection, otherwise returns distance
fn intersect(origin: &Point, vec: Vector, cube: &Point) -> f32 {
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
    if tmax >= tmin {
        let dist = f32::sqrt(f32::powf(vec.x * tmin, 2.0) + f32::powf(vec.y * tmin, 2.0));
        return dist;
    } else {
        // TODO: Change this to a constant
        return -1.0;
    }
}
// TODO: Should we convert it to an int now, or later?
// TODO: Figure out wtf this does, I basically copied from Python code
fn distance_to_height(dist: f32, angle: &f32) -> f32 {
    return (HEIGHT as f32 - 50.0) / (dist * f32::cos(*angle));
}

/// Main
fn main() {
    let origin = Point { x: 0.0, y: 0.0 };
    let cube = Point { x: 1.0, y: 1.0 };
    let iterations = 20;
    let mut theta = (f32::consts::PI / 4.0) + (FOV / 2.0);
    let delta_theta = FOV / (iterations as f32);
    for _ in 0..iterations {
        let vector = angle_to_vec(&theta);
        println!("{}", intersect(&origin, vector, &cube));
        theta -= delta_theta;
    }
}
