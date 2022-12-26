use crate::ray::Ray;
use crate::surface::{Surface, SurfaceIntersection};

pub struct World {
    pub surfaces: Vec<Box<dyn Surface>>,
}

impl World {
    pub fn new() -> World {
        World { surfaces: Vec::new() }
    }
}

impl Surface for World {
    fn raycast(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<SurfaceIntersection> {
        let mut result = None;
        let mut t_nearest = t_max;

        for obj in &self.surfaces {
            if let Some(intersection) = obj.raycast(r, t_min, t_nearest) {
                t_nearest = intersection.t;
                result = Some(intersection);
            }
        }

        return result;
    }
}
