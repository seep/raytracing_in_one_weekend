use crate::ray::Ray;
use crate::scatter::Scatter;
use crate::surface::SurfaceIntersection;
use crate::util::{reflect, refract};
use glam::*;
use rand::*;

pub struct DielectricMaterial {
    index_of_refraction: f32,
}

impl DielectricMaterial {
    pub fn new(index_of_refraction: f32) -> DielectricMaterial {
        DielectricMaterial { index_of_refraction }
    }
}

impl Scatter for DielectricMaterial {
    fn scatter(&self, r: &Ray, intersection: &SurfaceIntersection) -> Option<(Vec3, Ray)> {
        let refraction_ratio =
            if intersection.facing { 1.0 / self.index_of_refraction } else { self.index_of_refraction };

        let r_direction_norm = r.direction.normalize();

        let cos_theta = intersection.normal.dot(-r_direction_norm).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let schlick_approx = reflectance(cos_theta, refraction_ratio);

        let scattered_direction = if cannot_refract || schlick_approx > thread_rng().gen() {
            reflect(r_direction_norm, intersection.normal) // cannot refract
        } else {
            refract(r_direction_norm, intersection.normal, refraction_ratio)
        };

        let scattered = Ray::new(intersection.p, scattered_direction);

        Some((Vec3::ONE, scattered))
    }
}

fn reflectance(cos_theta: f32, refraction_ratio: f32) -> f32 {
    let r = ((1.0 - refraction_ratio) / (1.0 + refraction_ratio)).powi(2);
    return r + (1.0 - r) * (1.0 - cos_theta).powi(5);
}
