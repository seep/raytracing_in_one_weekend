use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use glam::*;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use raytracing_in_one_weekend::camera::Camera;
use raytracing_in_one_weekend::materials::dielectric::DielectricMaterial;
use raytracing_in_one_weekend::materials::lambertian::LambertianMaterial;
use raytracing_in_one_weekend::materials::metal::MetalMaterial;
use raytracing_in_one_weekend::ray::Ray;
use raytracing_in_one_weekend::scatter::Scatter;
use raytracing_in_one_weekend::sphere::Sphere;
use raytracing_in_one_weekend::surface::Surface;
use raytracing_in_one_weekend::util::rand_on_unit_sphere;
use raytracing_in_one_weekend::world::World;

fn background(ray: &Ray) -> Vec3 {
    const COLOR_T: Vec3 = Vec3::new(0.5, 0.7, 1.0);
    const COLOR_B: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    let ray_dir_normalized = ray.direction.normalize();

    let t = 0.5 * (ray_dir_normalized.y as f32 + 1.0);

    return Vec3::lerp(COLOR_B, COLOR_T, t);
}

fn raycast(world: &World, ray: &Ray, depth: u32) -> Vec3 {
    if depth <= 0 {
        return Vec3::ZERO;
    }

    return if let Some(intersection) = world.raycast(&ray, 0.001, f32::MAX) {
        if let Some((attenuation, scattered)) = intersection.material.scatter(ray, &intersection) {
            attenuation * raycast(&world, &scattered, depth - 1)
        } else {
            Vec3::ZERO
        }
    } else {
        background(&ray)
    };
}

fn create_world() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    world.surfaces.push({
        let mat = Arc::new(LambertianMaterial::new(Vec3::new(0.5, 0.5, 0.5)));
        let obj = Sphere::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0, mat);
        Box::new(obj)
    });

    for a in -11..11 {
        for b in -11..11 {
            let choose: f32 = rng.gen();

            let mat = if choose < 0.8 {
                let albedo = rand_on_unit_sphere() * rand_on_unit_sphere();
                Arc::new(LambertianMaterial::new(albedo)) as Arc<dyn Scatter>
            } else if choose < 0.95 {
                let albedo = Vec3::splat(0.4) + rand_on_unit_sphere() * 0.6;
                let fuzz = rng.gen_range(0.0..0.5);
                Arc::new(MetalMaterial::new(albedo, fuzz)) as Arc<dyn Scatter>
            } else {
                Arc::new(DielectricMaterial::new(1.5)) as Arc<dyn Scatter>
            };

            let center = Vec3::new((a as f32) + rng.gen_range(0.0..0.9), 0.2, (b as f32) + rng.gen_range(0.0..0.9));

            let obj = Sphere::new(center, 0.2, mat);

            world.surfaces.push(Box::new(obj));
        }
    }

    world.surfaces.push({
        let mat = Arc::new(DielectricMaterial::new(1.5));
        let obj = Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, mat);
        Box::new(obj)
    });

    world.surfaces.push({
        let mat = Arc::new(LambertianMaterial::new(Vec3::new(0.4, 0.2, 0.1)));
        let obj = Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0, mat);
        Box::new(obj)
    });

    world.surfaces.push({
        let mat = Arc::new(MetalMaterial::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
        let obj = Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, mat);
        Box::new(obj)
    });

    return world;
}

fn main() {
    const ASPECT_RATIO: f32 = 3.0 / 2.0;
    const IMAGE_W: u32 = 400;
    const IMAGE_H: u32 = (IMAGE_W as f32 / ASPECT_RATIO) as u32;

    const SAMPLES_PER_PIXEL: u32 = 20;
    const DEPTH: u32 = 5;

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

    write_ppm_header(&mut w, UVec2::new(IMAGE_W, IMAGE_H));

    // render all pixels in parallel

    let mut pixels = Vec::new();

    for y in (0..IMAGE_H).rev() {
        for x in 0..IMAGE_W {
            pixels.push(UVec2::new(x, y));
        }
    }

    let render = |pixel: UVec2| -> Vec3 {
        return sample_pixel(&world, &camera, pixel, UVec2::new(IMAGE_W, IMAGE_H), SAMPLES_PER_PIXEL, DEPTH);
    };

    let colors: Vec<Vec3> = pixels.into_par_iter().map(render).collect();

    for color in colors {
        // multisample averaging and gamma correction
        let r = (color.x / SAMPLES_PER_PIXEL as f32).sqrt().clamp(0.0, 0.999);
        let g = (color.y / SAMPLES_PER_PIXEL as f32).sqrt().clamp(0.0, 0.999);
        let b = (color.z / SAMPLES_PER_PIXEL as f32).sqrt().clamp(0.0, 0.999);
        write_ppm_color(&mut w, Vec3::new(r, g, b));
    }
}

fn sample_pixel(world: &World, camera: &Camera, p: UVec2, size: UVec2, samples: u32, depth: u32) -> Vec3 {
    let mut result = Vec3::new(0.0, 0.0, 0.0);

    // random multisampling
    for _ in 0..samples {
        let mut rng = rand::thread_rng();
        let u = (p.x as f32 + rng.gen_range(0.0..1.0)) / (size.x - 1) as f32;
        let v = (p.y as f32 + rng.gen_range(0.0..1.0)) / (size.y - 1) as f32;
        let r = camera.create_ray(u, v);
        result += raycast(&world, &r, depth);
    }

    return result;
}

pub fn write_ppm_header(w: &mut File, size: UVec2) {
    writeln!(w, "P3").unwrap();
    writeln!(w, "{} {}", size.x, size.y).unwrap();
    writeln!(w, "255").unwrap();
}

pub fn write_ppm_color(w: &mut File, color: Vec3) {
    writeln!(w, "{}", format_color(color)).unwrap()
}

fn format_color(color: Vec3) -> String {
    let r = (color.x * 255.999).clamp(0.0, 256.0) as i32;
    let g = (color.y * 255.999).clamp(0.0, 256.0) as i32;
    let b = (color.z * 255.999).clamp(0.0, 256.0) as i32;
    return format!("{} {} {}", r, g, b);
}
