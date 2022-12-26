use std::sync::Arc;

use crate::ray::Ray;
use crate::scatter::Scatter;
use glam::*;

pub trait Surface: Send + Sync {
    fn raycast(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<SurfaceIntersection>;
}

pub struct SurfaceIntersection {
    pub p: Vec3,
    pub normal: Vec3,
    pub facing: bool,
    pub material: Arc<dyn Scatter>,
    pub t: f32,
}
