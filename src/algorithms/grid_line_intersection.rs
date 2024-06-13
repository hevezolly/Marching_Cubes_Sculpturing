use glam::{IVec3, Vec3};
use num::clamp;

use super::{cordinates::RoundableToIVec3, raycast::{ray_box_intersection, resolve_ray_box_intersection, Ray}};

pub struct GridLineIntersection 
{
    step: IVec3,
    field_max: IVec3,
    field_min: IVec3,
    t_max: Vec3,
    current_position: IVec3,
    t_delta: Vec3,
    is_finished: bool,
}

impl Iterator for GridLineIntersection {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {return None;};
        
        let result = self.current_position;
        
        if self.t_max.x < self.t_max.y {
            if self.t_max.x < self.t_max.z {
                self.current_position.x += self.step.x;
                if self.current_position.x > self.field_max.x ||
                   self.current_position.x < self.field_min.x {
                    self.is_finished = true;
                }
                self.t_max.x += self.t_delta.x;
            }
            else {
                self.current_position.z += self.step.z;
                if self.current_position.z > self.field_max.z ||
                   self.current_position.z < self.field_min.z {
                    self.is_finished = true;
                }
                self.t_max.z += self.t_delta.z;
            }
        }
        else {
            if self.t_max.y < self.t_max.z {
                self.current_position.y += self.step.y;
                if self.current_position.y > self.field_max.y ||
                   self.current_position.y < self.field_min.y {
                    self.is_finished = true;
                }
                self.t_max.y += self.t_delta.y;
            }
            else {
                self.current_position.z += self.step.z;
                if self.current_position.z > self.field_max.z ||
                   self.current_position.z < self.field_min.z {
                    self.is_finished = true;
                }
                self.t_max.z += self.t_delta.z;
            }
        };
        Some(result)
    }
}

fn sign(val: f32) -> i32 {
    if val == 0. {0}
    else if val < 0. {-1}
    else {1}
}

fn inv_lerp(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

pub fn march_grid_by_ray<T: Into<IVec3>>(
    ray_origin: Vec3,
    ray_direction: Vec3,
    grid_bounds_min: T,
    grid_bounds_max: T,
) -> Option<GridLineIntersection> {
    let min_i: IVec3 = grid_bounds_min.into();
    let max_i: IVec3 = grid_bounds_max.into();

    let min = min_i.as_vec3();
    let max = max_i.as_vec3() + Vec3::ONE;

    let intersection = ray_box_intersection(min, max, Ray::new(ray_origin, ray_direction));
    let (closest_f, _) = resolve_ray_box_intersection(intersection, ray_origin, 
        min, max)?;

    let mut closest = closest_f.floor_to_ivec();
    closest.x = clamp(closest.x, min_i.x, max_i.x);
    closest.y = clamp(closest.y, min_i.y, max_i.y);
    closest.z = clamp(closest.z, min_i.z, max_i.z);
    let step = IVec3::new(sign(ray_direction.x), sign(ray_direction.y), sign(ray_direction.z));
    
    let mut next_cord = closest_f.floor() + Vec3::ONE;
    let ceiled = closest_f.ceil();
    if step.x < 0 {next_cord.x = ceiled.x - 1.}
    if step.y < 0 {next_cord.y = ceiled.y - 1.}
    if step.z < 0 {next_cord.z = ceiled.z - 1.}


    let t = Vec3::new(
        inv_lerp(next_cord.x, ray_origin.x, ray_origin.x + ray_direction.x),
        inv_lerp(next_cord.y, ray_origin.y, ray_origin.y + ray_direction.y),
        inv_lerp(next_cord.z, ray_origin.z, ray_origin.z + ray_direction.z),
    );
    let dir_len = ray_direction.length();
    let delta = Vec3::new(
        step.x as f32 / ray_direction.x,
        step.y as f32 / ray_direction.y,
        step.z as f32 / ray_direction.z,
    );

    Some(GridLineIntersection { 
        step, 
        field_max: max_i, 
        field_min: min_i, 
        t_max: t, 
        current_position: closest, 
        t_delta: delta, 
        is_finished: false, 
    })
}

