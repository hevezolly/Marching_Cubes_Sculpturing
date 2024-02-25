use core::buffers::buffer::BoundBufferContext;
use core::buffers::buffer::IndexBuffer;
use core::buffers::buffer::ShaderStorageBuffer;
use core::buffers::buffer::Usage;
use core::buffers::buffer::VertexBuffer;
use core::buffers::vertex_attributes::VertexArrayObject;
use core::shaders::shader::Shader;
use core::textures::image_provider::Image;
use core::textures::image_provider::ImageFormat;
use core::textures::texture::FilterMode;
use core::textures::texture::MipMapFilterMode;
use core::textures::texture::Texture;
use core::textures::texture::TextureAccess;
use core::textures::texture::WrapMode;
use core::textures::TextureUnit;
use core::GL;
use core::buffers::buffer::Buffer;
use core::shaders::shader_programm::ShaderProgramm;
use std::cmp::max;
use std::cmp::min;
// use core::shaders::Shader;
use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;

use egui_glfw_gl::egui::Color32;
use egui_glfw_gl::egui::CtxRef;
use egui_glfw_gl::egui::Pos2;
use egui_glfw_gl::egui::Stroke;
use egui_glfw_gl::gl;
use egui_glfw_gl::egui;
use glam::ivec3;
use glam::vec4;
use glam::IVec2;
use glam::IVec3;
use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::vec2;
use glam::vec3;
use glam::Vec4;
use glam::Vec4Swizzles;

use crate::algorithms::camera::Camera;
use crate::algorithms::camera::perspective::PerspectiveCamera;
use crate::application::triangulation_table::produce_triangulation_buffer;


#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/display_tex3d.vert", 
              "resources/shader_sources/display_tex3d.frag")]
struct ShaderUniforms {
    view: Mat4,
    projection: Mat4,
    image: TextureUnit,
    slice: f32,
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/fill_circle.compute")]
struct FillTextureUniforms {
    #[name("imgOutput")]
    img_output: TextureUnit,
    // imgOutput: TextureUnit
}   

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/marching_cubes.compute")]
struct MarchingCubesUniforms {
    #[name("scalarField")]
    scalar_field: TextureUnit,
    origin_offset: Vec3,
    field_scale: Vec3,
    num_boxes: IVec3,
    surface_level: f32,
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/display_model.vert", 
              "resources/shader_sources/display_model.frag")]
struct ModelDisplayUniform {
    view: Mat4,
    projection: Mat4,
    light_direction: Vec3
}


pub struct ExecutrionLogick {
    mode_vertex_buffer: VertexBuffer<ModelVertex>,
    camera: PerspectiveCamera,
    model_programm: ShaderProgramm,
    slice: f32,
    vertecies_count: i32,
    // image: Image
    // programm: ShaderProgramm,
}

const NUM_OF_CUBES: IVec3 = IVec3 { x: 32, y: 32, z: 32 };
const TEXTURE_DIM: IVec3 = IVec3 { 
    x: NUM_OF_CUBES.x + 2, 
    y: NUM_OF_CUBES.y + 2, 
    z: NUM_OF_CUBES.z + 2 
};

#[repr(C)]
#[derive(VertexDef)]
struct DefaultVertex {
    psition: Vec3,
    uv: Vec2
}

#[repr(C)]
#[derive(VertexDef)]
struct ModelVertex {
    position: Vec3,
    normal: Vec3
}

impl ExecutrionLogick {
    pub fn init() -> ExecutrionLogick {
        
        let positions = [
            DefaultVertex { psition: vec3(-1., -1., 1.), uv: vec2(0., 0.) },
            DefaultVertex { psition: vec3( 1., -1., 1.), uv: vec2(1., 0.) },
            DefaultVertex { psition: vec3( 1.,  1., 1.), uv: vec2(1., 1.) },
            DefaultVertex { psition: vec3(-1.,  1., 1.), uv: vec2(0., 1.) },
        ];

        let indecies = [
            0, 1, 2, 0, 2, 3u32
        ];

        let vertex_buffer = VertexBuffer::new();
        vertex_buffer.bind()
            .new_data(&positions, Usage::static_draw())
            .unbind();

        let index_buffer = IndexBuffer::new();
        index_buffer.bind()
            .new_data(&indecies, Usage::static_draw())
            .unbind();

        let model_vertecies_buffer = ShaderStorageBuffer::new();
        model_vertecies_buffer.bind()
            .empty::<[Vec3;3]>(
                (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z * 5) as usize,
                Usage::dynamic_copy()
            )
            .unbind();
        let count_buffer = ShaderStorageBuffer::new();
        count_buffer.bind()
            .new_data(&[0], Usage::dynamic_read())
            .unbind();
    
        let triangulation_buffer = produce_triangulation_buffer();

        // let image = Image::from_file(PathBuf::new()
        //     .join("resources")
        //     .join("images")
        //     .join("free-image.jpg")).unwrap();

        let mut texture = Texture::new_3d()
            .magnification_filter(FilterMode::Linear)
            .minification_filter(FilterMode::Linear)
            .wrap_mode_x(WrapMode::Repeat)
            .wrap_mode_y(WrapMode::Repeat)
            .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, TEXTURE_DIM.z, ImageFormat {
                lod: 0,
                format: gl::RED,
                internal_format: gl::R32F,
                data_type: gl::FLOAT,
            });
            // .with_data(&image.to_gl(gl::RGB, gl::UNSIGNED_BYTE));

        let mut compute = ShaderProgramm::new()
            .attach_shader(Shader::compute()
                .from_file(PathBuf::new()
                    .join("resources")
                    .join("shader_sources")
                    .join("fill_circle.compute"))
                .unwrap())
            .build().unwrap();


        texture.bind_image(1, TextureAccess::Write);

        compute.bind().set_uniforms(FillTextureUniforms {
            img_output: 1.into(),
        }).unwrap();

        GL!(gl::DispatchCompute((TEXTURE_DIM.x) as u32, 
            (TEXTURE_DIM.y) as u32, 
            (TEXTURE_DIM.z) as u32));
        GL!(gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT));

        let model_programm = ShaderProgramm::new()
            .attach_shader(Shader::vertex()
                .from_file(PathBuf::new()
                    .join("resources")
                    .join("shader_sources")
                    .join("display_model.vert"))
                .unwrap())
            .attach_shader(Shader::fragment()
                .from_file(PathBuf::new()
                    .join("resources")
                    .join("shader_sources")
                    .join("display_model.frag"))
                .unwrap())
            .build().unwrap();

        let mut marching_cubes = ShaderProgramm::new()
            .attach_shader(Shader::compute()
                .from_file(PathBuf::new()
                    .join("resources")
                    .join("shader_sources")
                    .join("marching_cubes.compute"))
                .unwrap())
            .build().unwrap();

        marching_cubes.bind().set_uniforms(MarchingCubesUniforms { 
            scalar_field: 1.into(), 
            origin_offset: Vec3::ZERO, 
            field_scale: 3. * Vec3::ONE / NUM_OF_CUBES.as_vec3(), 
            num_boxes: NUM_OF_CUBES, 
            surface_level: 0.4 }
        ).unwrap()
        .set_buffer(&count_buffer, 1)
        .set_buffer(&model_vertecies_buffer, 2)
        .set_buffer(&triangulation_buffer, 3);

        GL!(gl::DispatchCompute((NUM_OF_CUBES.x) as u32, (NUM_OF_CUBES.y) as u32, (NUM_OF_CUBES.z) as u32));
        GL!(gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT));
            
        let mut vertecies_count = [0i32];

        count_buffer.bind().read_from_start_to(&mut vertecies_count).unbind();
        let vertecies_count = vertecies_count[0];

        // let mut vertecies = vec![Vec3::ZERO; triangles_count as usize * 3];
        // model_vertecies_buffer.bind().read_from_start_to(&mut vertecies).unbind();

        // dbg!(vertecies);

        dbg!(vertecies_count);

        let camera = PerspectiveCamera::new(90., 0.01, 100.);

        ExecutrionLogick { 
            camera, 
            slice: 0.,
            mode_vertex_buffer: model_vertecies_buffer.rebind(),
            model_programm,
            vertecies_count,
        }

    }

    pub fn on_frame_end(&mut self) {

    }

    fn draw_model(&mut self) 
    {
        self.mode_vertex_buffer.bind();

        self.model_programm.bind().set_uniforms(ModelDisplayUniform {
            view: self.camera.view_matrix(),
            projection: self.camera.projection_matrix(),
            light_direction: vec3(1., 1., -0.5).normalize()
        }).unwrap();

        // GL!(gl::DrawArrays(gl::TRIANGLES, self.offset * 3, self.draw_count * 3));
        GL!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertecies_count));

        self.mode_vertex_buffer.unbind();
        self.model_programm.unbind();
    }

    pub fn draw(&mut self, params: Parameters) {

        GL!(gl::Enable(gl::DEPTH_TEST));
        GL!(gl::ClearColor(0.455, 0.302, 0.663, 1.0));
        GL!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        
        
        self.camera.set_aspect_ratio(params.height as f32 / params.width as f32);

        // self.draw_quad();
        self.draw_model();
    }

    fn draw_box(&self, egui_ctx: &CtxRef, corner: Vec3, size: Vec3, screen_size: Vec2) {
        let painter = egui_ctx.debug_painter();
        let corner = vec4(corner.x, corner.y, corner.z, 1.);

        // let size = vec2(screen_size.x, screen_size.y) / egui_ctx.pixels_per_point();
        //  egui_ctx.pixels_per_point();

        let lbb = corner;
        let lbf = corner + Vec4::Z * size.z;
        let ltb = corner + Vec4::Y * size.y;
        let ltf = corner + Vec4::Y * size.y + Vec4::Z * size.z;
        let rbb = corner + Vec4::X * size.x;
        let rbf = corner + Vec4::X * size.x + Vec4::Z * size.z;
        let rtb = corner + Vec4::X * size.x + Vec4::Y * size.y;
        let rtf = corner + Vec4::X * size.x + Vec4::Y * size.y + Vec4::Z * size.z;

        let lbb = self.camera.full_matrix() * lbb;
        let lbf = self.camera.full_matrix() * lbf;
        let ltb = self.camera.full_matrix() * ltb;
        let ltf = self.camera.full_matrix() * ltf;
        let rbb = self.camera.full_matrix() * rbb;
        let rbf = self.camera.full_matrix() * rbf;
        let rtb = self.camera.full_matrix() * rtb;
        let rtf = self.camera.full_matrix() * rtf;

        let lbb = ((lbb.xy() / lbb.w) + Vec2::ONE) * 0.5;
        let lbf = ((lbf.xy() / lbf.w) + Vec2::ONE) * 0.5;
        let ltb = ((ltb.xy() / ltb.w) + Vec2::ONE) * 0.5;
        let ltf = ((ltf.xy() / ltf.w) + Vec2::ONE) * 0.5;
        let rbb = ((rbb.xy() / rbb.w) + Vec2::ONE) * 0.5;
        let rbf = ((rbf.xy() / rbf.w) + Vec2::ONE) * 0.5;
        let rtb = ((rtb.xy() / rtb.w) + Vec2::ONE) * 0.5;
        let rtf = ((rtf.xy() / rtf.w) + Vec2::ONE) * 0.5;

        let lbb = Pos2::new(lbb.x * screen_size.x, (1. - lbb.y) * screen_size.y);
        let lbf = Pos2::new(lbf.x * screen_size.x, (1. - lbf.y) * screen_size.y);
        let ltb = Pos2::new(ltb.x * screen_size.x, (1. - ltb.y) * screen_size.y);
        let ltf = Pos2::new(ltf.x * screen_size.x, (1. - ltf.y) * screen_size.y);
        let rbb = Pos2::new(rbb.x * screen_size.x, (1. - rbb.y) * screen_size.y);
        let rbf = Pos2::new(rbf.x * screen_size.x, (1. - rbf.y) * screen_size.y);
        let rtb = Pos2::new(rtb.x * screen_size.x, (1. - rtb.y) * screen_size.y);
        let rtf = Pos2::new(rtf.x * screen_size.x, (1. - rtf.y) * screen_size.y);

        let stroke = Stroke::new(1., Color32::GREEN);

        painter.line_segment([lbb, rbb], stroke);
        painter.line_segment([ltb, lbb], stroke);
        painter.line_segment([rtb, rbb], stroke);
        painter.line_segment([ltb, rtb], stroke);

        painter.line_segment([lbf, rbf], stroke);
        painter.line_segment([ltf, lbf], stroke);
        painter.line_segment([rtf, rbf], stroke);
        painter.line_segment([ltf, rtf], stroke);

        painter.line_segment([lbb, lbf], stroke);
        painter.line_segment([rbb, rbf], stroke);
        painter.line_segment([rtb, rtf], stroke);
        painter.line_segment([ltb, ltf], stroke);

    }

    pub fn draw_ui(&mut self, egui_ctx: &CtxRef, params: Parameters) {
        egui::Window::new("Settings").show(egui_ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.slice, 0.0..=1.0).text("slice"));
            let mut scale = self.camera.transform.scale().x;

            ui.add(egui::DragValue::new(&mut scale).speed(0.1));
            self.camera.transform.set_scale(vec3(scale, scale, scale));

            const speed: f32 = 0.2;

            // let mut offset = self.offset;
            // let mut count = self.draw_count;
            // if ui.add(egui::Slider::new(&mut offset, 0..=(self.triangles_count - 1))
            //     .text("offset"))
            //     .changed() {
            //     count = min(count, self.triangles_count - offset);
            // }

            // if ui.add(egui::Slider::new(&mut count, 1..=self.triangles_count)
            //     .text("count"))
            //     .changed() {
            //     offset = min(offset, self.triangles_count - count);
            // }

            // self.offset = offset;
            // self.draw_count = count;

            let mut fov = self.camera.fov();
            ui.add(egui::Slider::new(&mut fov, 1.0..=179.).text("fov"));
            self.camera.set_fov(fov);

            if egui_ctx.input().key_pressed(egui::Key::S) {
                let pos = self.camera.transform.position() + vec3(0., 0., -speed);
                self.camera.transform.set_position(pos);
            }
            if egui_ctx.input().key_pressed(egui::Key::W) {
                let pos = self.camera.transform.position() + vec3(0., 0., speed);
                self.camera.transform.set_position(pos);
            }
            if egui_ctx.input().key_pressed(egui::Key::A) {
                let pos = self.camera.transform.position() + vec3(-speed, 0., 0.);
                self.camera.transform.set_position(pos);
            }
            if egui_ctx.input().key_pressed(egui::Key::D) {
                let pos = self.camera.transform.position() + vec3(speed, 0., 0.);
                self.camera.transform.set_position(pos);
            }
            if egui_ctx.input().key_pressed(egui::Key::Space) {
                let pos = self.camera.transform.position() + vec3(0., speed, 0.);
                self.camera.transform.set_position(pos);
            }
            if egui_ctx.input().key_pressed(egui::Key::Z) {
                let pos = self.camera.transform.position() + vec3(0., -speed, 0.);
                self.camera.transform.set_position(pos);
            }

            // let size = vec2(params.width as f32, params.height as f32) / egui_ctx.pixels_per_point();

            // let scale = Vec3::ONE;

            // for x in 0..NUM_OF_CUBES.x {
            //     for y in 0..NUM_OF_CUBES.y {
            //         for z in 0..NUM_OF_CUBES.z {
            //             let position = ivec3(x, y, z).as_vec3() * scale;
            //             self.draw_box(egui_ctx, position, scale, size);
            //         }
            //     }
            // } 
        });
    }
}

pub struct Parameters {
    pub width: i32,
    pub height: i32
}