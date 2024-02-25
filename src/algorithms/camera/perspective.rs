
use glam::{Vec3, Mat4, Vec4Swizzles, vec4};

use crate::algorithms::{raycast::Ray, transform::Transform};

use super::Camera;

const DEG_TO_RAD: f32 = 3.14159265 / 180.;

pub struct PerspectiveCamera{
    pub transform: Transform,
    perspective_matrix: Mat4,
    fov: f32,
    znear: f32,
    zfar: f32,
    aspect_ratio: f32,
}

impl PerspectiveCamera {
    pub fn new(fov: f32, znear: f32, zfar: f32) -> PerspectiveCamera {
        let mut result = PerspectiveCamera {
            transform: Transform::IDENTITY,
            fov,
            znear,
            zfar,
            aspect_ratio: 1.,
            perspective_matrix: Mat4::IDENTITY
        };
        result.recalculate_perspective_matrix();
        result
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.recalculate_perspective_matrix();
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.recalculate_perspective_matrix();
    }

    fn recalculate_perspective_matrix(&mut self) {
        let f = 1.0 / (self.fov * DEG_TO_RAD / 2.0).tan();
        let m33 = (self.zfar+self.znear)/(self.zfar-self.znear);
        let m34= -(2.0*self.zfar*self.znear)/(self.zfar-self.znear);
        self.perspective_matrix = Mat4::from_cols_array(
            &[f * self.aspect_ratio, 0.0, 0.0, 0.0,
                0.0,              f,   0.0, 0.0,
                0.0,              0.0, m33, m34,
                0.0,              0.0, 1.0, 0.0]
        ).transpose()
    }
}


impl Camera for PerspectiveCamera {
    fn viewport_point_to_ray(&self, viewport_point: Vec3) -> Ray {
        let p_inv = self.perspective_matrix.inverse();
        let v_inv = self.transform.matrix();

        let point = Vec3::new((viewport_point.x - 0.5) * 2., (viewport_point.y - 0.5) * 2., -1.);
        let point4 = vec4(point.x, point.y, point.z, 1.);

        let mut dir_eye = p_inv * point4;
        dir_eye.w = 0.;

        let dir_world = (v_inv * dir_eye).xyz();

        
        Ray::new(self.transform.position(), dir_world.normalize())
    }

    fn view_matrix(&self) -> Mat4 {
        self.transform.matrix().inverse()
    }

    fn projection_matrix(&self) -> Mat4 {
        self.perspective_matrix
    }
}