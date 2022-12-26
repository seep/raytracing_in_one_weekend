use crate::ray::Ray;
use crate::surface::SurfaceIntersection;
use glam::*;

pub trait Scatter: Send + Sync {
    fn scatter(&self, r: &Ray, intersection: &SurfaceIntersection) -> Option<(Vec3, Ray)>;
}
