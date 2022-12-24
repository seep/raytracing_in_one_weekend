use std::fs::File;
use std::io::Write;
use std::path::Path;
use glam::{Vec3, DVec3};

mod sphere;

use sphere::{Sphere};

type Color = DVec3;

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub const fn new(origin: Vec3, direction: Vec3) -> Ray { Ray { origin, direction } }
    pub fn at(&self, t: f32) -> Vec3 { self.origin + self.direction * t }
}

struct Intersection {
    p: Vec3,
    normal: Vec3,
    facing: bool,
    t: f32,
}

trait Intersect {
    fn intersect(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<Intersection>;
}

pub type World = Vec<Box<dyn Intersect>>;

fn find_closest_intersection(world: &World, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
    let mut result = None;
    let mut t_nearest = t_max;

    for obj in world {
        if let Some(intersection) = obj.intersect(ray, t_min, t_nearest) {
            t_nearest = intersection.t;
            result = Some(intersection);
        }
    }

    return result;
}

fn background(ray: &Ray) -> Color {
    const COLOR_T: Color = Color::new(0.5, 0.7, 1.0);
    const COLOR_B: Color = Color::new(1.0, 1.0, 1.0);

    let ray_dir_normalized = ray.direction.normalize();

    let t = 0.5 * (ray_dir_normalized.y as f64 + 1.0);

    return DVec3::lerp(COLOR_B, COLOR_T, t);
}

fn scene(world: &World, ray: &Ray) -> Color {
    if let Some(intersection) = find_closest_intersection(world, ray, 0.0, f32::MAX) {
        return ((intersection.normal + Vec3::ONE) * 0.5).as_dvec3();
    } else {
        return background(&ray);
    }
}

fn main() {
    let path = Path::new("image.ppm");
    let mut w = File::create(&path).unwrap();

    writeln!(&mut w, "P3").unwrap();
    writeln!(&mut w, "{} {}", IMAGE_W, IMAGE_H).unwrap();
    writeln!(&mut w, "255").unwrap();

    const ASPECT_RATIO: f32 = 16.0 / 9.0;
    const IMAGE_W: u64 = 400;
    const IMAGE_H: u64 = (IMAGE_W as f32 / ASPECT_RATIO) as u64;

    let mut world = World::new();
    world.push(Box::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)));
    world.push(Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)));

    let viewport_h = 2.0 as f32;
    let viewport_w = (viewport_h * ASPECT_RATIO) as f32;
    let focal_length = 1.0;

    let origin = Vec3::ZERO;
    let scan_h = Vec3::new(viewport_w, 0.0, 0.0);
    let scan_w = Vec3::new(0.0, viewport_h, 0.0);
    let llc = origin - (scan_h * 0.5) - (scan_w * 0.5) - Vec3::new(0.0, 0.0, focal_length);

    for y in (0..IMAGE_H).rev() {
        println!("Scanlines remaining: {}", y);
        for x in (0..IMAGE_W).rev() {
            let u = x as f32 / (IMAGE_W - 1) as f32;
            let v = y as f32 / (IMAGE_H - 1) as f32;
            let r = Ray::new(origin, llc + scan_h * u + scan_w * v - origin);
            let c = scene(&world, &r);
            writeln!(&mut w, "{}", format_color(c)).unwrap();
        }
    }
}

fn format_color(color: Color) -> String {
    let r = (color.x * 255.999) as i32;
    let g = (color.y * 255.999) as i32;
    let b = (color.z * 255.999) as i32;
    return format!("{} {} {}", r, g, b);
}
