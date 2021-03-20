use image::GenericImageView;
use anyhow::*;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler
}

impl Texture {
    pub fn from_bytes(device: &wgpu::Device, queue:&wgpu::Queue, bytes :&[u8],label:&str) -> Result<Self> {
        // make an image
        // get an image representation
        let img = image::load_from_memory(bytes)?;

        // call from image
        Self::from_image(device, queue, img, label)
    }
    pub fn from_image(device: &wgpu::Device, queue:&wgpu::Queue, img: &image::DynamicImage,label:&str) -> Result<Self> {

        // convert to vec? more complex actually ImageBuffer<RGBA<u8>, Vec<u8>>
        let rgba = img.as_rgba8()?;
        // get the dimensions of the image
        let dimensions = img.dimensions();

        // making the actual texture
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        // diffuse texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // sampled means we want to use it in our shaders, like how we defined them as sampler2D
            // also if we want to copy data into the texture
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label,
        });
        // use the queue to put data in the texture
        // can't put the data in the texture using the other referenc
        queue.write_texture(
            //
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO, // I guess it treats textures as 3D so you have to say write to the texture at 0 in 3D
            },
            // use the binary rgba data for our texture
            rgba,
            // specify a layout, haha everything is layouts
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * dimensions.0, // this is 4 times the width of our image, has to be a multiple of 256 apparently
                rows_per_image: dimensions.1,
            },
            // provide the actual size
            size,
        );

        // make a view

        let view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());
        // and a sampler
        // this is where you say whether it should read around edges or whatnot
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // these have to do with when a fragment covers multiple pixels
            // or if multiple fragments for single pixel
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Ok(Self {
            sampler,
            view,
            texture
        })
    }
}
