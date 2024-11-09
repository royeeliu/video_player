use std::{
    rc::Rc,
    sync::{mpsc, Arc},
};
use winit::{dpi::PhysicalSize, window::Window};

extern crate ffmpeg_next as ffmpeg;
use ffmpeg::software::scaling;

use crate::wgpu_context::WgpuContext;
use crate::{presenter::Presenter, texture::Texture};
type VideoReceiver = mpsc::Receiver<ffmpeg::frame::Video>;

pub struct VideoRenderer {
    video_receiver: VideoReceiver,
    context: Option<Rc<WgpuContext>>,
    presenter: Option<Presenter>,
}

impl VideoRenderer {
    pub fn new(video_receiver: VideoReceiver) -> Self {
        VideoRenderer {
            video_receiver,
            context: None,
            presenter: None,
        }
    }

    pub fn init(&mut self, window: Arc<Window>) {
        let instance = wgpu::Instance::default();
        println!("WGPU instance created.");
        let surface = instance.create_surface(window.clone()).unwrap();
        let context = Rc::new(WgpuContext::new(&instance, &surface));
        self.context = Some(context.clone());

        let size = window.inner_size();
        self.presenter = Some(Presenter::new(
            context.clone(),
            surface,
            size.width,
            size.height,
        ));
    }

    pub fn render(&mut self) {
        if let (Some(context), Some(presenter)) = (self.context.as_ref(), self.presenter.as_mut()) {
            if let Some(yuv_frame) = self.video_receiver.recv().ok() {
                let mut scaler = scaling::Context::get(
                    yuv_frame.format(),
                    yuv_frame.width(),
                    yuv_frame.height(),
                    ffmpeg::util::format::Pixel::RGBA,
                    yuv_frame.width(),
                    yuv_frame.height(),
                    scaling::Flags::BILINEAR,
                )
                .unwrap();

                let mut rgb_frame = ffmpeg::frame::Video::empty();
                scaler.run(&yuv_frame, &mut rgb_frame).unwrap();

                let data = rgb_frame.data(0);

                let texture =
                    Texture::new(&context.device, (yuv_frame.width(), yuv_frame.height())).unwrap();

                context.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        aspect: wgpu::TextureAspect::All,
                        texture: &texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    &data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(rgb_frame.stride(0) as u32),
                        rows_per_image: Some(rgb_frame.height()),
                    },
                    texture.texture.size(),
                );

                presenter.draw(&texture);
            }
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(context) = self.presenter.as_mut() {
            context.resize(size.width, size.height);
        }
    }
}
