use glam::{Mat4, Vec3};

use crate::algorithms::raycast::Ray;

pub mod perspective;

pub trait Camera {
    fn view_matrix(&self) -> Mat4;
    fn projection_matrix(&self) -> Mat4;

    fn full_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    fn viewport_point_to_ray(&self, screen_point: Vec3) -> Ray;
}