use std::arch::x86_64::__cpuid;

use glam::{IVec3, Vec2, Vec3, Vec3Swizzles};

use super::Triangle;

// use crate::field::coordinates::{Cord, PartialComponents};

// use super::triangulation::Triangle;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self { 
        Self { origin, direction: direction.normalize() } 
    }
}

#[derive(Debug)]
pub struct RaycastResult {
    pub chunk_cord: IVec3,
    pub cord_in_chunk: (usize, usize, usize),
    pub intersection_point: Vec3,
}

#[derive(Debug)]
pub enum IntersectionResult {
    None,
    Single(Vec3),
    Double(Vec3, Vec3)
}

fn ray_plane_intersection(position: Vec3, normal: Vec3, ray: Ray) -> Option<Vec3> {
    let to_plane = position - ray.origin;
    let to_plane_proj = normal * Vec3::dot(normal, to_plane);
    let dir_proj_dot = Vec3::dot(to_plane_proj, ray.direction);
    if dir_proj_dot <= 0. {
        None
    }
    else {
        let plane_ort = normal * Vec3::dot(normal, ray.direction);
        let plane_par = (ray.direction - plane_ort) * to_plane_proj.length() / plane_ort.length();
        Some(ray.origin + to_plane_proj + plane_par)
    }
}

pub fn ray_triangle_intersection(ray: Ray, triangle: &Triangle) -> Option<Vec3> {
    let plane_position = triangle.a();
    let plane_normal = triangle.normal();
    if let Some(plane_hit) = ray_plane_intersection(plane_position, plane_normal, ray) {
        // return Some(plane_hit);
        let e1 = Vec3::cross(triangle.ab(), plane_hit - triangle.a()).normalize();
        let e2 = Vec3::cross(triangle.bc(), plane_hit - triangle.b()).normalize();
        let e3 = Vec3::cross(triangle.ca(), plane_hit - triangle.c()).normalize();

        let e1 = Vec3::dot(e1, plane_normal) >= 0.;
        let e2 = Vec3::dot(e2, plane_normal) >= 0.;
        let e3 = Vec3::dot(e3, plane_normal) >= 0.;

        if (e1 == e2) && (e2 == e3) && (e1 == e3) {
            Some(plane_hit)
        }
        else {
            None
        }
    }
    else {
        None
    }
}

fn inside_rect(rect_min: Vec2, rect_size: Vec2, point: Vec2) -> bool {
    point.x >= rect_min.x && point.x <= rect_min.x + rect_size.x && point.y >= rect_min.y && point.y <= rect_min.y + rect_size.y
} 

#[derive(Debug)]
struct MinSize {
    min: Vec3,
    size: Vec2
}

#[derive(Debug)]
enum Plane {
    XY(MinSize),
    ZY(MinSize),
    XZ(MinSize),
}

impl Plane {

    fn is_inside(&self, point: Vec3) -> bool {
        match &self {
            Plane::XY(m) => inside_rect(m.min.xy(), m.size, point.xy()),
            Plane::ZY(m) => inside_rect(m.min.zy(), m.size, point.zy()),
            Plane::XZ(m) => inside_rect(m.min.xz(), m.size, point.xz()),
        }
    }

    fn intersection(&self, ray: Ray) -> Option<Vec3> {
        match &self {
            Plane::XY(m) => ray_plane_intersection(m.min, Vec3::Z, ray),
            Plane::ZY(m) => ray_plane_intersection(m.min, Vec3::X, ray),
            Plane::XZ(m) => ray_plane_intersection(m.min, Vec3::Y, ray),
        }
    }

    fn intersect_and_iside(&self, ray: Ray) -> Option<Vec3> {
        self.intersection(ray).filter(|p| self.is_inside(*p))
    }
}

pub fn ray_box_intersection(b_min: Vec3, b_max: Vec3, ray: Ray) -> IntersectionResult {
    let mut results:Vec<Vec3> = Vec::new();
    let size = b_max - b_min;

    for plane in [
        Plane::XY(MinSize { min: Vec3::new(b_min.x, b_min.y, b_min.z), size: Vec2::new(size.x, size.y) }),
        Plane::XY(MinSize { min: Vec3::new(b_min.x, b_min.y, b_max.z), size: Vec2::new(size.x, size.y) }),
        Plane::XZ(MinSize { min: Vec3::new(b_min.x, b_min.y, b_min.z), size: Vec2::new(size.x, size.z) }),
        Plane::XZ(MinSize { min: Vec3::new(b_min.x, b_max.y, b_min.z), size: Vec2::new(size.x, size.z) }),
        Plane::ZY(MinSize { min: Vec3::new(b_min.x, b_min.y, b_min.z), size: Vec2::new(size.z, size.y) }),
        Plane::ZY(MinSize { min: Vec3::new(b_max.x, b_min.y, b_min.z), size: Vec2::new(size.z, size.y) }),
    ] {
        if let Some(intersection) = plane.intersect_and_iside(ray) {
            match results[..] {
                [] => results.push(intersection),
                [val, ..] if Vec3::distance_squared(val, intersection) < 0.001 => break,
                [val, ..] if Vec3::distance(val, ray.origin) < Vec3::distance(intersection, ray.origin) => {
                    results.push(intersection);
                    break;
                },
                [val, ..] => {
                    results = vec![intersection, val];
                    break;
                },
            };
        }
    };

    if results.len() == 0 {
        IntersectionResult::None
    }
    else if results.len() == 1 {
        IntersectionResult::Single(results[0])
    }
    else {
        IntersectionResult::Double(results[0], results[1])
    }
}


pub fn resolve_ray_box_intersection(
    intersection: IntersectionResult, 
    origin: Vec3, 
    bbox_min: Vec3, 
    bbox_max: Vec3) -> Option<(Vec3, Vec3)> 
{
    match intersection {
        IntersectionResult::None => None,
        IntersectionResult::Single(point) => resolve_single_intersection(
            point, origin, bbox_min, bbox_max),
        IntersectionResult::Double(c, f) => Some((c, f)),
    }
}

fn resolve_single_intersection(
    intersection: Vec3, 
    origin: Vec3, 
    bbox_min: Vec3, 
    bbox_max: Vec3
) -> Option<(Vec3, Vec3)> 
{
    if intersection.x >= bbox_min.x && intersection.x <= bbox_max.x &&
       intersection.y >= bbox_min.y && intersection.y <= bbox_max.y &&
       intersection.z >= bbox_min.z && intersection.z <= bbox_max.z {
        Some((origin, intersection))
    }
    else {
        None
    }
}