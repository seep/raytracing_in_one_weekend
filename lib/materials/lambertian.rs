use crate::ray::Ray;
use crate::scatter::Scatter;
use crate::surface::SurfaceIntersection;
use crate::util::{is_near_zero, rand_on_unit_sphere};
use glam::*;

pub struct LambertianMaterial {
    albedo: Vec3,
}

impl LambertianMaterial {
    pub fn new(albedo: Vec3) -> LambertianMaterial {
        LambertianMaterial { albedo }
    }
}

impl Scatter for LambertianMaterial {
    fn scatter(&self, _r: &Ray, intersection: &SurfaceIntersection) -> Option<(Vec3, Ray)> {
        let mut scattered_direction = intersection.normal + rand_on_unit_sphere();

        if is_near_zero(scattered_direction) {
            scattered_direction = intersection.normal
        }

        let scattered = Ray::new(intersection.p, scattered_direction);

        return Some((self.albedo, scattered));
    }
}
