use std::{ffi::{c_void, CString}, path::Path, ptr::NonNull, slice::from_raw_parts};
use stb_image::stb_image;
use ::stb_image::stb_image::{stbi_load, stbi_set_flip_vertically_on_load};


#[derive(Default, Debug, Clone, Copy)]
pub struct ImageFormat {
    pub lod: i32,
    pub format: u32,
    pub internal_format: u32,
    pub data_type: u32,
}

pub trait ImageProvider2D {
    fn description(&self) -> ImageFormat;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn data(&self) -> *const c_void;
}

#[derive(Debug, Clone)]
pub struct Image {
    image: Box<[u8]>,
    width: i32,
    height: i32,
}

pub struct GlImage<'a> {
    image: &'a Image,
    format: u32,
    data_type: u32,
}

impl Image {
    pub fn from_file<P: AsRef<Path>>(path: P) ->
        Result<Image, String> 
    {
        let path = path.as_ref().canonicalize()
            .map_err(|e| format!("{e}"))?;
            
        let path = path.to_str().ok_or("Failed to convert path to string")?;
        let c_path = CString::new(path).map_err(|e| format!("{e}"))?;
        let mut width = 0;
        let mut height = 0;
        let mut num_chanels = 0;
        let image_data = unsafe {
            stbi_set_flip_vertically_on_load(1);
            stbi_load(c_path.as_ptr(), &mut width, &mut height, &mut num_chanels, 0)
        }; 

        let image_data = NonNull::new(image_data).ok_or("image is null")?;

        let image_data = unsafe {
            from_raw_parts::<u8>(image_data.as_ptr(), (width * height * num_chanels) as usize)
        };

        let img = Image {
            image: image_data.into(),
            width,
            height,
        };
        
        Ok(img)
    }

    pub fn to_gl(&self, format: u32, data_type: u32) -> GlImage {
        GlImage { image: self, format, data_type }
    }
}

impl<'a> ImageProvider2D for GlImage<'a> {
   

    fn data(&self) -> *const c_void {
        self.image.image.as_ptr() as *const c_void
    }

    fn description(&self) -> ImageFormat {
        ImageFormat { 
            lod: 0, 
            format: self.format, 
            internal_format: self.format, 
            data_type: self.data_type }
    }

    fn width(&self) -> i32 {
        self.image.width
    }

    fn height(&self) -> i32 {
        self.image.height
    }
}


