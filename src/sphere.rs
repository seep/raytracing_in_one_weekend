use glam::Vec3;
use std::rc::Rc;

use super::Ray;
use super::Scatter;
use super::Surface;
use super::SurfaceIntersection;

pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Rc<dyn Scatter>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Rc<dyn Scatter>) -> Sphere {
        return Sphere { center, radius, material };
    }
}

impl Surface for Sphere {
    fn raycast(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<SurfaceIntersection> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - (self.radius * self.radius);

        let discriminant = (half_b * half_b) - (a * c);

        if discriminant < 0.0 {
            return None;
        }

        let discriminant_sqrt = discriminant.sqrt();

        let root_lower = (-half_b - discriminant_sqrt) / a;
        let root_upper = (-half_b + discriminant_sqrt) / a;

        let mut root = root_lower;

        if root < t_min || t_max < root {
            root = root_upper;
            if root < t_max || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = r.at(t);

        let outward_normal = (p - self.center) / self.radius;
        let facing = r.direction.dot(outward_normal) < 0.0;
        let normal = if facing { outward_normal } else { -outward_normal };

        return Some(SurfaceIntersection { p, t, facing, normal, material: self.material.clone() });
    }
}
