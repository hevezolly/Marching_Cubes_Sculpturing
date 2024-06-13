use std::sync::{Arc, Mutex, MutexGuard};

use crate::algorithms::camera::Camera;



#[derive(Clone)]
pub struct CameraRef<T: Camera> {
    cam: Arc<Mutex<T>>,
}

#[derive(Clone)]
pub struct CameraRefDyn {
    cam: Arc<Mutex<dyn Camera>>
}

impl<Cam: Camera> Camera for CameraRef<Cam> {
    fn view_matrix(&self) -> glam::Mat4 {
        self.cam.lock().unwrap().view_matrix()
    }

    fn projection_matrix(&self) -> glam::Mat4 {
        self.cam.lock().unwrap().projection_matrix()
    }

    fn viewport_point_to_ray(&self, screen_point: glam::Vec3) -> crate::algorithms::raycast::Ray {
        self.cam.lock().unwrap().viewport_point_to_ray(screen_point)
    }
}

impl Camera for CameraRefDyn {
    fn view_matrix(&self) -> glam::Mat4 {
        self.cam.lock().unwrap().view_matrix()
    }

    fn projection_matrix(&self) -> glam::Mat4 {
        self.cam.lock().unwrap().projection_matrix()
    }

    fn viewport_point_to_ray(&self, screen_point: glam::Vec3) -> crate::algorithms::raycast::Ray {
        self.cam.lock().unwrap().viewport_point_to_ray(screen_point)
    }
}

impl<Cam: Camera + 'static> CameraRef<Cam>  {

    pub fn new(cam: Cam) -> Self {
        Self { cam: Arc::new(Mutex::new(cam)) }
    }

    pub fn get(&mut self) -> MutexGuard<'_, Cam> {
        self.cam.lock().unwrap()
    }

    pub fn clone_dyn(&self) -> CameraRefDyn {
        CameraRefDyn { cam: self.cam.clone() }
    }
}
