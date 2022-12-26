use glam::*;
use rand::thread_rng;
use rand_distr::*;

pub fn is_near_zero(v: Vec3) -> bool {
    return v.abs_diff_eq(Vec3::ZERO, f32::EPSILON);
}

pub fn rand_in_unit_disc() -> Vec2 {
    return Vec2::from(UnitDisc.sample(&mut thread_rng()));
}

pub fn rand_in_unit_sphere() -> Vec3 {
    return Vec3::from(UnitBall.sample(&mut thread_rng()));
}

pub fn rand_on_unit_sphere() -> Vec3 {
    return Vec3::from(UnitSphere.sample(&mut thread_rng()));
}

pub fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
    return v - (2.0 * v.dot(normal) * normal);
}

pub fn refract(v: Vec3, normal: Vec3, ratio: f32) -> Vec3 {
    let inv_normal = -normal;
    let r_perp = (v + v.dot(inv_normal).min(1.0) * normal) * ratio;
    let r_para = (1.0 - r_perp.length_squared()).abs().sqrt() * inv_normal;
    return r_perp + r_para;
}
