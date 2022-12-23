use glam::{Vec3};

use super::Ray;
use super::Intersect;
use super::Intersection;

pub struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Sphere { Sphere { center, radius } }
}

impl Intersect for Sphere {
    fn intersect(&self, r: &Ray, t_min: f32, t_max:f32) -> Option<Intersection> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = Vec3::dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let discriminant_sqrt = f32::sqrt(discriminant);

        let root_lower = (-half_b - discriminant_sqrt) / a;
        let root_upper = (-half_b + discriminant_sqrt) / a;

        let t: f32;

        if t_min <= root_lower && root_lower <= t_max {
            t = root_lower;
        } else if t_min <= root_upper && root_upper <= t_max {
            t = root_upper;
        } else {
            return None;
        }

        let p = r.at(t);
        let normal = (p - self.center) / self.radius;
        let facing = r.direction.dot(normal) < 0.0;

        return Some(Intersection { p, t, facing, normal });
    }
}