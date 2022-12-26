use glam::*;

use crate::ray::Ray;
use crate::util::rand_in_unit_disc;

pub struct Camera {
    pub origin: Vec3,
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
