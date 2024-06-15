use glam::{Vec3};

pub mod camera;
pub mod raycast;
pub mod transform;
pub mod grid_line_intersection;
pub mod cordinates;

#[derive(Debug, Clone, Copy)]
pub struct Triangle(Vec3, Vec3, Vec3);

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Triangle {
        Triangle(a, b, c)
    }

    pub fn a(&self) -> Vec3 {
        self.0
    }

    pub fn b(&self) -> Vec3 {
        self.1
    }

    pub fn c(&self) -> Vec3 {
        self.2
    }

    pub fn ab(&self) -> Vec3 {
        self.b() - self.a()
    }

    pub fn bc(&self) -> Vec3 {
        self.c() - self.b()
    }

    pub fn ca(&self) -> Vec3 {
        self.a() - self.c()
    }

    pub fn ac(&self) -> Vec3 {
        self.c() - self.a()
    }

    pub fn normal(&self) -> Vec3 {
        Vec3::cross(self.ab(), self.ac()).normalize()
    }

    pub fn offset_triangle(&self, offset: f32) -> Triangle {

        let a_ang = Vec3::angle_between(self.ab(), self.ac());
        let a_dist = offset / f32::sin(a_ang);

        let a = self.a() - (self.ab().normalize() + self.ac().normalize()) * a_dist;

        let b_ang = Vec3::angle_between(-self.ab(), self.bc());
        let b_dist = offset / f32::sin(b_ang);

        let b = self.b() - (-self.ab().normalize() + self.bc().normalize()) * b_dist;

        let c_ang = Vec3::angle_between(self.ca(), -self.bc());
        let c_dist = offset / f32::sin(c_ang);

        let c = self.c() - (self.ca().normalize() - self.bc().normalize()) * c_dist;
        
        Triangle(a, b, c)
    }
}
