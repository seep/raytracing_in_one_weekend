use crate::ray::Ray;
use crate::scatter::Scatter;
use crate::surface::SurfaceIntersection;
use crate::util::{rand_in_unit_sphere, reflect};
use glam::*;

pub struct MetalMaterial {
    albedo: Vec3,
    fuzz: f32,
}

impl MetalMaterial {
    pub fn new(albedo: Vec3, fuzz: f32) -> MetalMaterial {
        MetalMaterial { albedo, fuzz }
    }
}

impl Scatter for MetalMaterial {
    fn scatter(&self, r: &Ray, intersection: &SurfaceIntersection) -> Option<(Vec3, Ray)> {
        let reflected_direction = reflect(r.direction, intersection.normal).normalize();
        let scattered_direction = reflected_direction + rand_in_unit_sphere() * self.fuzz;
        let scattered = Ray::new(intersection.p, scattered_direction);

        return if scattered.direction.dot(intersection.normal) > 0.0 { Some((self.albedo, scattered)) } else { None };
    }
}
