use std::fs::File;
use std::io::Write;
use std::path::Path;
use glam::{Vec2, Vec3, DVec3};
use rand_distr::{UnitSphere, Distribution};
use rand::Rng;

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

struct Camera {
    origin: Vec3,
    viewport: Vec2,
    focal_length: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        let viewport_h = 2.0 as f32;
        let viewport_w = (viewport_h * aspect_ratio) as f32;
        let viewport = Vec2::new(viewport_w, viewport_h);

        let focal_length = 1.0;

        let origin = Vec3::ZERO;

        return Camera { origin, viewport, focal_length }
    }

    pub fn create_ray(&self, uv: Vec2) -> Ray {
        let view_uv = self.viewport * uv;
        let lower_left_corner = self.origin - Vec3::new(self.viewport.x * 0.5, self.viewport.y * 0.5, self.focal_length);

        Ray::new(self.origin, lower_left_corner + Vec3::new(view_uv.x, view_uv.y, 0.0) - self.origin)
    }
}

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

fn scene(world: &World, ray: &Ray, depth: u64) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(intersection) = find_closest_intersection(world, ray, 0.0, f32::MAX) {
        // sample a random point on the tangent sphere of the intersection, then cast a new
        // ray from the intersection point through the random point
        let random_sphere_center = intersection.p + intersection.normal;
        let random_sphere_sample = Vec3::from_array(UnitSphere.sample(&mut rand::thread_rng()));
        let bounce_direction = (random_sphere_center + random_sphere_sample) - intersection.p;
        let bounce = Ray::new(intersection.p, bounce_direction.normalize());
        return 0.5 * scene(&world, &bounce, depth - 1);
    } else {
        return background(&ray)
    }
}

fn main() {
    const ASPECT_RATIO: f32 = 16.0 / 9.0;
    const IMAGE_W: u64 = 400;
    const IMAGE_H: u64 = (IMAGE_W as f32 / ASPECT_RATIO) as u64;

    let path = Path::new("image.ppm");
    let mut w = File::create(&path).unwrap();

    writeln!(&mut w, "P3").unwrap();
    writeln!(&mut w, "{} {}", IMAGE_W, IMAGE_H).unwrap();
    writeln!(&mut w, "255").unwrap();

    let camera = Camera::new(ASPECT_RATIO);

    let mut world = World::new();
    world.push(Box::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)));
    world.push(Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)));

    const SAMPLES_PER_PIXEL: u16 = 100;
    const DEPTH: u64 = 5;

    let mut rng = rand::thread_rng();

    for y in (0..IMAGE_H).rev() {
        for x in (0..IMAGE_W).rev() {
            let mut c = Color::new(0.0, 0.0, 0.0);

            for _ in 0..SAMPLES_PER_PIXEL {
                let rand_u: f32 = rng.gen();
                let rand_v: f32 = rng.gen();

                let u = (x as f32 + rand_u) / (IMAGE_W as f32 - 1.0);
                let v = (y as f32 + rand_v) / (IMAGE_H as f32 - 1.0);
                let uv = Vec2::new(u, v);

                c += scene(&world, &camera.create_ray(uv), DEPTH);
            }

            // multisample averaging
            c /= SAMPLES_PER_PIXEL as f64;

            // gamma correction
            c.x = c.x.sqrt().clamp(0.0, 0.999);
            c.y = c.y.sqrt().clamp(0.0, 0.999);
            c.z = c.z.sqrt().clamp(0.0, 0.999);

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
