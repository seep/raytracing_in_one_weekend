use glam::{DVec3, Vec2, Vec3};
use rand::Rng;
use rand_distr::{Distribution, UnitBall, UnitDisc, UnitSphere};

use std::fs::File;
use std::io::Write;
use std::ops::Neg;
use std::path::Path;
use std::sync::Arc;

mod sphere;

use sphere::Sphere;

type Color = DVec3;

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub const fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

pub struct SurfaceIntersection {
    p: Vec3,
    normal: Vec3,
    facing: bool,
    material: Arc<dyn Scatter>,
    t: f32,
}

pub trait Surface: Send + Sync {
    fn raycast(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<SurfaceIntersection>;
}

pub trait Scatter: Send + Sync {
    fn scatter(&self, r: &Ray, intersection: &SurfaceIntersection) -> Option<(Color, Ray)>;
}

pub struct LambertianMaterial {
    albedo: Color,
}

impl LambertianMaterial {
    pub fn new(albedo: Color) -> LambertianMaterial {
        LambertianMaterial { albedo }
    }
}

impl Scatter for LambertianMaterial {
    fn scatter(&self, _r: &Ray, intersection: &SurfaceIntersection) -> Option<(Color, Ray)> {
        let mut scattered_direction = intersection.normal + rand_on_unit_sphere();

        if is_near_zero(scattered_direction) {
            scattered_direction = intersection.normal
        }

        let scattered = Ray::new(intersection.p, scattered_direction);

        return Some((self.albedo, scattered));
    }
}

pub struct MetalMaterial {
    albedo: Color,
    fuzz: f32,
}

impl MetalMaterial {
    pub fn new(albedo: Color, fuzz: f32) -> MetalMaterial {
        MetalMaterial { albedo, fuzz }
    }
}

impl Scatter for MetalMaterial {
    fn scatter(&self, r: &Ray, intersection: &SurfaceIntersection) -> Option<(Color, Ray)> {
        let reflected_direction = reflect(r.direction, intersection.normal).normalize();
        let scattered_direction = reflected_direction + rand_in_unit_sphere() * self.fuzz;
        let scattered = Ray::new(intersection.p, scattered_direction);

        return if scattered.direction.dot(intersection.normal) > 0.0 { Some((self.albedo, scattered)) } else { None };
    }
}

pub struct DielectricMaterial {
    index_of_refraction: f32,
}

impl DielectricMaterial {
    pub fn new(index_of_refraction: f32) -> DielectricMaterial {
        DielectricMaterial { index_of_refraction }
    }
}

impl Scatter for DielectricMaterial {
    fn scatter(&self, r: &Ray, intersection: &SurfaceIntersection) -> Option<(Color, Ray)> {
        let refraction_ratio =
            if intersection.facing { 1.0 / self.index_of_refraction } else { self.index_of_refraction };

        let r_direction_norm = r.direction.normalize();

        let cos_theta = intersection.normal.dot(-r_direction_norm).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let schlick_approx = reflectance(cos_theta, refraction_ratio);

        let scattered_direction = if cannot_refract || schlick_approx > rand::thread_rng().gen() {
            reflect(r_direction_norm, intersection.normal) // cannot refract
        } else {
            refract(r_direction_norm, intersection.normal, refraction_ratio)
        };

        let scattered = Ray::new(intersection.p, scattered_direction);

        Some((Color::ONE, scattered))
    }
}

fn reflectance(cos_theta: f32, refraction_ratio: f32) -> f32 {
    let r = ((1.0 - refraction_ratio) / (1.0 + refraction_ratio)).powi(2);
    return r + (1.0 - r) * (1.0 - cos_theta).powi(5);
}

fn is_near_zero(v: Vec3) -> bool {
    return v.abs_diff_eq(Vec3::ZERO, f32::EPSILON);
}

fn rand_in_unit_disc() -> Vec2 {
    return Vec2::from(UnitDisc.sample(&mut rand::thread_rng()));
}

fn rand_in_unit_sphere() -> Vec3 {
    return Vec3::from(UnitBall.sample(&mut rand::thread_rng()));
}

fn rand_on_unit_sphere() -> Vec3 {
    return Vec3::from(UnitSphere.sample(&mut rand::thread_rng()));
}

fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
    return v - (2.0 * v.dot(normal) * normal);
}

fn refract(v: Vec3, normal: Vec3, ratio: f32) -> Vec3 {
    let inv_normal = normal.neg();
    let r_perp = (v + v.dot(inv_normal).min(1.0) * normal) * ratio;
    let r_para = (1.0 - r_perp.length_squared()).abs().sqrt() * inv_normal;
    return r_perp + r_para;
}

type World = Vec<Box<dyn Surface>>;

struct Camera {
    origin: Vec3,
    llc: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    cu: Vec3,
    cv: Vec3,
    aperture: f32,
}

impl Camera {
    pub fn new(
        origin: Vec3,
        target: Vec3,
        up: Vec3,
        vertial_fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focal_length: f32,
    ) -> Camera {
        let theta = std::f32::consts::PI / 180.0 * vertial_fov;

        let viewport_h = 2.0 * (theta * 0.5).tan();
        let viewport_w = viewport_h * aspect_ratio;

        let cw = (origin - target).normalize();
        let cu = up.cross(cw).normalize();
        let cv = cw.cross(cu);

        let h = focal_length * viewport_w * cu;
        let v = focal_length * viewport_h * cv;

        let llc = origin - (h * 0.5) - (v * 0.5) - focal_length * cw;

        return Camera { origin, llc, horizontal: h, vertical: v, cu, cv, aperture };
    }

    pub fn create_ray(&self, s: f32, t: f32) -> Ray {
        let rand_in_lens_disc = rand_in_unit_disc() * self.aperture * 0.5;
        let offset = self.cu * rand_in_lens_disc.x + self.cv * rand_in_lens_disc.y;

        return Ray::new(
            self.origin + offset,
            self.llc + s * self.horizontal + t * self.vertical - self.origin - offset,
        );
    }
}

fn find_closest_intersection(world: &World, ray: &Ray, t_min: f32, t_max: f32) -> Option<SurfaceIntersection> {
    let mut result = None;
    let mut t_nearest = t_max;

    for obj in world {
        if let Some(intersection) = obj.raycast(ray, t_min, t_nearest) {
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

fn raycast(world: &World, ray: &Ray, depth: u64) -> Color {
    if depth <= 0 {
        return Color::ZERO;
    }

    return if let Some(intersection) = find_closest_intersection(world, ray, 0.001, f32::MAX) {
        if let Some((attenuation, scattered)) = intersection.material.scatter(ray, &intersection) {
            attenuation * raycast(&world, &scattered, depth - 1)
        } else {
            Color::ZERO
        }
    } else {
        background(&ray)
    };
}

fn create_world() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    world.push({
        let mat = Arc::new(LambertianMaterial::new(Color::new(0.5, 0.5, 0.5)));
        let obj = Sphere::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0, mat);
        Box::new(obj)
    });

    for a in -11..11 {
        for b in -11..11 {
            let choose: f32 = rng.gen();

            let mat = if choose < 0.8 {
                let albedo = (rand_on_unit_sphere() * rand_on_unit_sphere()).as_dvec3();
                Arc::new(LambertianMaterial::new(albedo)) as Arc<dyn Scatter>
            } else if choose < 0.95 {
                let albedo = (Vec3::splat(0.4) + rand_on_unit_sphere() * 0.6).as_dvec3();
                let fuzz = rng.gen_range(0.0..0.5);
                Arc::new(MetalMaterial::new(albedo, fuzz)) as Arc<dyn Scatter>
            } else {
                Arc::new(DielectricMaterial::new(1.5)) as Arc<dyn Scatter>
            };

            let center = Vec3::new((a as f32) + rng.gen_range(0.0..0.9), 0.2, (b as f32) + rng.gen_range(0.0..0.9));

            let obj = Sphere::new(center, 0.2, mat);

            world.push(Box::new(obj));
        }
    }

    world.push({
        let mat = Arc::new(DielectricMaterial::new(1.5));
        let obj = Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, mat);
        Box::new(obj)
    });

    world.push({
        let mat = Arc::new(LambertianMaterial::new(Color::new(0.4, 0.2, 0.1)));
        let obj = Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0, mat);
        Box::new(obj)
    });

    world.push({
        let mat = Arc::new(MetalMaterial::new(Color::new(0.7, 0.6, 0.5), 0.0));
        let obj = Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, mat);
        Box::new(obj)
    });

    return world;
}

fn main() {
    const ASPECT_RATIO: f32 = 3.0 / 2.0;
    const IMAGE_W: u64 = 400;
    const IMAGE_H: u64 = (IMAGE_W as f32 / ASPECT_RATIO) as u64;

    const SAMPLES_PER_PIXEL: u64 = 20;
    const DEPTH: u64 = 5;

    let world = create_world();

    let camera_origin = Vec3::new(13.0, 2.0, 3.0);
    let camera_target = Vec3::new(0.0, 0.0, 0.0);
    let camera_vertical_fov = 20.0;
    let camera_focal_length = 10.0;
    let camera_aperture = 0.1;

    let camera = Camera::new(
        camera_origin,
        camera_target,
        Vec3::Y,
        camera_vertical_fov,
        ASPECT_RATIO,
        camera_aperture,
        camera_focal_length,
    );

    let path = Path::new("image.ppm");
    let mut w = File::create(&path).unwrap();

    writeln!(&mut w, "P3").unwrap();
    writeln!(&mut w, "{} {}", IMAGE_W, IMAGE_H).unwrap();
    writeln!(&mut w, "255").unwrap();

    for y in (0..IMAGE_H).rev() {
        println!("Scanline {}", y);
        for x in 0..IMAGE_W {
            let mut c = Color::new(0.0, 0.0, 0.0);

            // random multisampling
            for _ in 0..SAMPLES_PER_PIXEL {
                let mut rng = rand::thread_rng();
                let rand_u: f32 = rng.gen();
                let rand_v: f32 = rng.gen();

                let u = (x as f32 + rand_u) / (IMAGE_W - 1) as f32;
                let v = (y as f32 + rand_v) / (IMAGE_H - 1) as f32;
                let ray = camera.create_ray(u, v);

                c += raycast(&world, &ray, DEPTH);
            }

            // multisample averaging and gamma correction
            let r = (c.x / SAMPLES_PER_PIXEL as f64).sqrt().clamp(0.0, 0.999);
            let g = (c.y / SAMPLES_PER_PIXEL as f64).sqrt().clamp(0.0, 0.999);
            let b = (c.z / SAMPLES_PER_PIXEL as f64).sqrt().clamp(0.0, 0.999);

            writeln!(&mut w, "{}", format_color(Color::new(r, g, b))).unwrap();
        }
    }
}

fn format_color(color: Color) -> String {
    let r = (color.x * 255.999) as i32;
    let g = (color.y * 255.999) as i32;
    let b = (color.z * 255.999) as i32;
    return format!("{} {} {}", r, g, b);
}
