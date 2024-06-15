use std::{rc::Rc, sync::{Arc, Mutex}};

use egui_glfw_gl::egui::{Color32, CtxRef, Pos2, Stroke};
use glam::{vec2, vec4, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use num::traits::bounds;

use crate::{algorithms::{camera::Camera, cordinates::WComp, Triangle}, application::cunks::marching_cubes::MarchParameters};

use super::{bounds::Bounds, camera_ref::CameraRefDyn};


const ENABLE_DEBUG: bool = false;

pub enum DebugPrimitive {
    Box{
        corner: Vec3,
        size: Vec3,
    },
    Point(Vec3),
    Triangle(Triangle)
}

impl DebugPrimitive {
    pub fn Bounds(bounds: &Bounds<Vec3>) -> DebugPrimitive {
        DebugPrimitive::Box { corner: bounds.min(), size: bounds.size() }
    }
}

struct DebugDraw {
    primitive: DebugPrimitive,
    color: Color32,
    matrix: Mat4,
    width: f32,
}

impl DebugDraw {
    fn draw(&self, camera: &impl Camera, egui_ctx: &CtxRef) {
        match self.primitive {
            DebugPrimitive::Box { corner, size } => draw_box(camera, egui_ctx, self.matrix, corner, size, self.color, self.width),
            DebugPrimitive::Point(p) => draw_point(self.matrix, self.color, p, egui_ctx, camera),
            DebugPrimitive::Triangle(t) => draw_triangle(camera, egui_ctx, self.matrix, self.color, t, self.width),
        }
    } 
}

#[derive(Clone)]
pub struct Debugger {
    matrix: Mat4,
    primitives: Arc<Mutex<Vec<DebugDraw>>>
}

fn draw_point(matrix: Mat4, color: Color32, center: Vec3, egui_ctx: &CtxRef, camera: &impl Camera) {
    let screen_size = screen_size(egui_ctx);
    let painter = egui_ctx.debug_painter();
    let center = vec4(center.x, center.y, center.z, 1.);
    let center = camera.full_matrix() * matrix * center;
    let scale = 10. / center.w;
    if scale < 0. {
        return;
    }
    let center = ((center.xy() / center.w) + Vec2::ONE) * 0.5;
    let center = Pos2::new(center.x * screen_size.x, (1. - center.y) * screen_size.y);

    painter.circle_filled(center, scale, color);
}

#[inline]
fn screen_size(egui_ctx: &CtxRef) -> Vec2 {
    vec2(egui_ctx.input().screen_rect.size().x, egui_ctx.input().screen_rect.size().y)
}

fn draw_triangle(camera: &impl Camera, egui_ctx: &CtxRef, matrix: Mat4, color: Color32, triangle: Triangle, width: f32) {
    let painter = egui_ctx.debug_painter();
    let screen_size = screen_size(egui_ctx);
    //  egui_ctx.pixels_per_point();

    let a = triangle.a().w1();
    let b = triangle.b().w1();
    let c = triangle.c().w1();

    let m = camera.full_matrix() * matrix; 

    let a = m * a;
    let b = m * b;
    let c = m * c;

    let a = ((a.xyz() / a.w) + Vec3::ONE) * 0.5;
    let b = ((b.xyz() / b.w) + Vec3::ONE) * 0.5;
    let c = ((c.xyz() / c.w) + Vec3::ONE) * 0.5;

    let draw_a = a.z > -1. && a.z < 1.;
    let draw_b = b.z > -1. && b.z < 1.;
    let draw_c = c.z > -1. && c.z < 1.;

    let a = Pos2::new(a.x * screen_size.x, (1. - a.y) * screen_size.y);
    let b = Pos2::new(b.x * screen_size.x, (1. - b.y) * screen_size.y);
    let c = Pos2::new(c.x * screen_size.x, (1. - c.y) * screen_size.y);

    let stroke = Stroke::new(width, color);


    if draw_a && draw_b {
        painter.line_segment([a, b], stroke);
    }
    if draw_b && draw_c {
        painter.line_segment([b, c], stroke);
    }
    if draw_c && draw_a {
        painter.line_segment([c, a], stroke);
    }
}

fn draw_box(
    camera: &impl Camera,
    egui_ctx: &CtxRef,
    matrix: Mat4,
    corner: Vec3, 
    size: Vec3, 
    color: Color32, 
    width: f32) {
    let painter = egui_ctx.debug_painter();
    let corner = vec4(corner.x, corner.y, corner.z, 1.);

    let screen_size = screen_size(egui_ctx);
    //  egui_ctx.pixels_per_point();

    let lbb = corner;
    let lbf = corner + Vec4::Z * size.z;
    let ltb = corner + Vec4::Y * size.y;
    let ltf = corner + Vec4::Y * size.y + Vec4::Z * size.z;
    let rbb = corner + Vec4::X * size.x;
    let rbf = corner + Vec4::X * size.x + Vec4::Z * size.z;
    let rtb = corner + Vec4::X * size.x + Vec4::Y * size.y;
    let rtf = corner + Vec4::X * size.x + Vec4::Y * size.y + Vec4::Z * size.z;

    let m = camera.full_matrix() * matrix; 

    let lbb = m * lbb;
    let lbf = m * lbf;
    let ltb = m * ltb;
    let ltf = m * ltf;
    let rbb = m * rbb;
    let rbf = m * rbf;
    let rtb = m * rtb;
    let rtf = m * rtf;

    
    
    let lbb = ((lbb.xyz() / lbb.w) + Vec3::ONE) * 0.5;
    let lbf = ((lbf.xyz() / lbf.w) + Vec3::ONE) * 0.5;
    let ltb = ((ltb.xyz() / ltb.w) + Vec3::ONE) * 0.5;
    let ltf = ((ltf.xyz() / ltf.w) + Vec3::ONE) * 0.5;
    let rbb = ((rbb.xyz() / rbb.w) + Vec3::ONE) * 0.5;
    let rbf = ((rbf.xyz() / rbf.w) + Vec3::ONE) * 0.5;
    let rtb = ((rtb.xyz() / rtb.w) + Vec3::ONE) * 0.5;
    let rtf = ((rtf.xyz() / rtf.w) + Vec3::ONE) * 0.5;

    let draw_lbb: bool = lbb.z > -1. && lbb.z < 1.;
    let draw_lbf = lbf.z > -1. && lbf.z < 1.;
    let draw_ltb = ltb.z > -1. && ltb.z < 1.;
    let draw_ltf = ltf.z > -1. && ltf.z < 1.;
    let draw_rbb = rbb.z > -1. && rbb.z < 1.;
    let draw_rbf = rbf.z > -1. && rbf.z < 1.;
    let draw_rtb = rtb.z > -1. && rtb.z < 1.;
    let draw_rtf = rtf.z > -1. && rtf.z < 1.;

    let lbb = Pos2::new(lbb.x * screen_size.x, (1. - lbb.y) * screen_size.y);
    let lbf = Pos2::new(lbf.x * screen_size.x, (1. - lbf.y) * screen_size.y);
    let ltb = Pos2::new(ltb.x * screen_size.x, (1. - ltb.y) * screen_size.y);
    let ltf = Pos2::new(ltf.x * screen_size.x, (1. - ltf.y) * screen_size.y);
    let rbb = Pos2::new(rbb.x * screen_size.x, (1. - rbb.y) * screen_size.y);
    let rbf = Pos2::new(rbf.x * screen_size.x, (1. - rbf.y) * screen_size.y);
    let rtb = Pos2::new(rtb.x * screen_size.x, (1. - rtb.y) * screen_size.y);
    let rtf = Pos2::new(rtf.x * screen_size.x, (1. - rtf.y) * screen_size.y);

    let stroke = Stroke::new(width, color);


    if draw_lbb && draw_rbb {
        painter.line_segment([lbb, rbb], stroke);
    }
    if draw_ltb && draw_lbb {
        painter.line_segment([ltb, lbb], stroke);
    }
    if draw_rtb && draw_rbb {
        painter.line_segment([rtb, rbb], stroke);
    }
    if draw_ltb && draw_rtb {
        painter.line_segment([ltb, rtb], stroke);
    }
    if draw_lbf && draw_rbf {
        painter.line_segment([lbf, rbf], stroke);
    }
    if draw_ltf && draw_lbf {
        painter.line_segment([ltf, lbf], stroke);
    }
    if draw_rtf && draw_rbf {
        painter.line_segment([rtf, rbf], stroke);
    }
    if draw_ltf && draw_rtf {
        painter.line_segment([ltf, rtf], stroke);
    }
    if draw_lbb && draw_lbf {
        painter.line_segment([lbb, lbf], stroke);
    }
    if draw_rbb && draw_rbf {
        painter.line_segment([rbb, rbf], stroke);
    }
    if draw_rtb && draw_rtf {
        painter.line_segment([rtb, rtf], stroke);
    }
    if draw_ltb && draw_ltf {
        painter.line_segment([ltb, ltf], stroke);
    }

}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger { matrix: Mat4::IDENTITY, primitives: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn clone_with_matrix(&self, matrix: Mat4) -> Debugger {
        let mut d = self.clone();
        d.matrix = matrix;
        d
    }

    pub fn draw(&mut self, primitive: DebugPrimitive, color: Color32) {
        if !ENABLE_DEBUG {return;}
        self.primitives.lock().unwrap().push(DebugDraw {
            primitive,
            color,
            matrix: self.matrix,
            width: 1.
        });
    }

    pub fn draw_width(&mut self, primitive: DebugPrimitive, color: Color32, width: f32) {
        if !ENABLE_DEBUG {return;}
        self.primitives.lock().unwrap().push(DebugDraw {
            primitive,
            color,
            matrix: self.matrix,
            width
        });
    }

    pub fn perform_draw(&mut self, egui_ctx: &CtxRef, camera: &impl Camera) {
        let mut prim_lock = self.primitives.lock().unwrap();
        for d in prim_lock.iter() {
            d.draw(camera, egui_ctx);
        }

        prim_lock.clear()
    }
}



