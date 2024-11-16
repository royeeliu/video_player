#[allow(dead_code)]
pub(crate) struct WgpuContext {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuContext {
    pub fn new(instance: &wgpu::Instance, surface: &wgpu::Surface<'_>) -> Self {
        pollster::block_on(Self::new_async(instance, surface))
    }

    pub async fn new_async(instance: &wgpu::Instance, surface: &wgpu::Surface<'_>) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Request adapter failed");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Request device failed");

        WgpuContext {
            adapter,
            device,
            queue,
        }
    }

    pub fn write_texture(
        &self,
        texture: &wgpu::Texture,
        data: &[u8],
        pitch: u32,
        width: u32,
        height: u32,
    ) {
        assert!(data.len() >= (pitch * height) as usize);
        let size = texture.size();
        assert_eq!(size.width, width);
        assert_eq!(size.height, height);

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
