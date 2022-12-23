use std::fs::File;
use std::io::Write;
use std::path::Path;
use glam::{Vec3, DVec3};

type Color = DVec3;

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    #[inline(always)]
    pub const fn new(origin: Vec3, direction: Vec3) -> Self { Self { origin, direction } }

    #[inline(always)]
    pub fn at(&self, t: f32) -> Vec3 { self.origin + self.direction * t }
}

fn background(ray: &Ray) -> Color {
    const COLOR_T: Color = Color::new(0.5, 0.7, 1.0);
    const COLOR_B: Color = Color::new(1.0, 1.0, 1.0);

    let ray_dir_normalized = ray.direction.normalize();

    let t = 0.5 * (ray_dir_normalized.y as f64 + 1.0);

    return DVec3::lerp(COLOR_B, COLOR_T, t);
}

fn sphere(center: Vec3, radius: f32, ray: &Ray) -> f32 {
    let oc = ray.origin - center;
    let a = Vec3::dot(ray.direction, ray.direction);
    let b = Vec3::dot(ray.direction, oc) * 2.0;
    let c = Vec3::dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    return if discriminant < 0.0 {
        -1.0
    } else {
        (-b - f32::sqrt(discriminant)) / (a * 2.0)
    }
}

fn scene(ray: &Ray) -> Color {
    let sphere_origin = Vec3::NEG_Z;
    let sphere_radius = 0.5;

    let t = sphere(sphere_origin, sphere_radius, &ray);

    if t > 0.0 {
        let t_point = ray.at(t);
        let t_normal = (t_point - sphere_origin).normalize();
        return ((t_normal + Vec3::ONE) * 0.5).as_dvec3();
    }

    return background(&ray);
}

fn main() {
    let aspect = 16.0 / 9.0;
    let image_w = 400;
    let image_h = (image_w as f32 / aspect) as i32;

    let viewport_h = 2.0;
    let viewport_w = viewport_h * aspect;
    let focal_length = 1.0;

    let origin = Vec3::ZERO;
    let scan_h = Vec3::new(viewport_w, 0.0, 0.0);
    let scan_w = Vec3::new(0.0, viewport_h, 0.0);
    let llc = origin - (scan_w * 0.5) - (scan_h * 0.5) - Vec3::new(0.0, 0.0, focal_length);

    let path = Path::new("image.ppm");
    let mut w = File::create(&path).unwrap();

    writeln!(&mut w, "P3").unwrap();
    writeln!(&mut w, "{} {}", image_w, image_h).unwrap();
    writeln!(&mut w, "255").unwrap();

    for y in (0..image_h).rev() {
        println!("Scanlines remaining: {}", y);
        for x in (0..image_w).rev() {
            let u = x as f32 / (image_w - 1) as f32;
            let v = y as f32 / (image_h - 1) as f32;
            let r = Ray::new(origin, llc + scan_h * u + scan_w * v - origin);
            write_color(&mut w, scene(&r));
        }
    }
}

fn write_color(file: &mut File, color: Color) {
    let r = (color.x * 255.999) as i32;
    let g = (color.y * 255.999) as i32;
    let b = (color.z * 255.999) as i32;
    writeln!(file, "{} {} {}", r, g, b).unwrap();
}