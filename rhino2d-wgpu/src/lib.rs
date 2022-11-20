mod node;

use std::{
    borrow::Cow,
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use image::ImageFormat;
use rhino2d_io::TextureEncoding;
use wgpu::{
    util::DeviceExt, BindGroup, Device, Extent3d, Queue, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};

pub struct Gpu {
    pub device: Device,
    pub queue: Queue,
}

pub struct Renderer {
    gpu: Gpu,
    textures: Vec<Texture>,
}

impl Renderer {
    pub fn new(gpu: Gpu, puppet: &rhino2d_io::InochiPuppet) -> io::Result<Self> {
        let mut textures = Vec::with_capacity(puppet.textures().len());
        for texture in puppet.textures() {
            let info = TextureInfo::new(texture)?;

            let texture = gpu.device.create_texture_with_data(
                &gpu.queue,
                &TextureDescriptor {
                    label: None,
                    size: info.extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: info.texture_format,
                    usage: TextureUsages::TEXTURE_BINDING,
                },
                &info.data,
            );
            textures.push(texture);
        }

        Ok(Self { gpu, textures })
    }
}

struct TextureInfo<'a> {
    data: Cow<'a, [u8]>,
    texture_format: TextureFormat,
    extent: Extent3d,
}

impl<'a> TextureInfo<'a> {
    fn new(texture: &rhino2d_io::Texture) -> io::Result<Self> {
        let width;
        let height;
        let mut tex_fmt = TextureFormat::Rgba8UnormSrgb;
        let data: Cow<[u8]> = match texture.encoding() {
            TextureEncoding::Png => {
                let image = image::load_from_memory_with_format(texture.data(), ImageFormat::Png)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .to_rgba8();
                width = image.width();
                height = image.height();
                image.into_vec().into()
            }
            TextureEncoding::Tga => {
                let image = image::load_from_memory_with_format(texture.data(), ImageFormat::Tga)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .to_rgba8();
                width = image.width();
                height = image.height();
                image.into_vec().into()
            }
            TextureEncoding::Bc7 => {
                // Inochi2D does not yet support this. The file format is missing required metadata
                // to load this type of texture (height and width).
                #[allow(unused_assignments)]
                {
                    tex_fmt = TextureFormat::Bc7RgbaUnormSrgb;
                }
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "BC7 textures are not yet supported",
                ));
            }
            unk => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{unk:?} textures are not yet supported"),
                ));
            }
        };

        Ok(Self {
            data,
            texture_format: tex_fmt,
            extent: Extent3d {
                width,
                height,
                ..Default::default()
            },
        })
    }
}
