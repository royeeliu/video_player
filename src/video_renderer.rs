use std::{
    mem::swap,
    rc::Rc,
    sync::{mpsc, Arc},
};
use winit::{dpi::PhysicalSize, window::Window};

extern crate ffmpeg_next as ffmpeg;
use ffmpeg::software::scaling;

use crate::wgpu_context::WgpuContext;
use crate::{presenter::Presenter, texture::Texture};
type VideoReceiver = mpsc::Receiver<ffmpeg::frame::Video>;
type VideoSender = mpsc::SyncSender<ffmpeg::frame::Video>;

pub struct VideoRenderer {
    video_receiver: VideoReceiver,
    context: Option<Rc<WgpuContext>>,
    presenter: Option<Presenter>,
    texture: Option<Texture>,
}

impl VideoRenderer {
    pub fn new(video_receiver: VideoReceiver) -> Self {
        VideoRenderer {
            video_receiver,
            context: None,
            presenter: None,
            texture: None,
        }
    }

    pub fn init(&mut self, window: Arc<Window>, request_redraw: Box<dyn Fn() + Send>) {
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

        let (sender, mut receiver) = mpsc::sync_channel(1);
        swap(&mut self.video_receiver, &mut receiver);

        std::thread::spawn(move || {
            Self::thread_loop(receiver, sender, request_redraw);
        });
    }

    pub fn render(&mut self) {
        if let (Some(context), Some(presenter)) = (self.context.as_ref(), self.presenter.as_mut()) {
            if let Some(frame) = self.video_receiver.try_recv().ok() {
                self.texture = Self::update_texture(
                    context,
                    self.texture.take(),
                    frame.width(),
                    frame.height(),
                );

                if let Some(texture) = self.texture.as_ref() {
                    let data = frame.data(0);
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
                            bytes_per_row: Some(frame.stride(0) as u32),
                            rows_per_image: Some(frame.height()),
                        },
                        texture.texture.size(),
                    );
                }
            }

            if let Some(texture) = self.texture.as_ref() {
                presenter.draw(&texture);
            }
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(context) = self.presenter.as_mut() {
            context.resize(size.width, size.height);
        }
    }

    fn update_texture(
        context: &WgpuContext,
        mut texture: Option<Texture>,
        width: u32,
        height: u32,
    ) -> Option<Texture> {
        let mut need_reset = false;
        if let Some(texture) = &texture {
            let tex_size = texture.texture.size();
            if (width, height) != (tex_size.width, tex_size.height) {
                need_reset = true;
            }
        }

        if need_reset {
            texture = None;
        }

        if texture.is_some() {
            texture
        } else {
            Some(Texture::new(&context.device, width, height).unwrap())
        }
    }

    fn thread_loop(
        receiver: VideoReceiver,
        sender: VideoSender,
        request_redraw: Box<dyn Fn() + Send>,
    ) {
        loop {
            if let Ok(frame) = receiver.recv() {
                let mut scaler = scaling::Context::get(
                    frame.format(),
                    frame.width(),
                    frame.height(),
                    ffmpeg::util::format::Pixel::RGBA,
                    frame.width(),
                    frame.height(),
                    scaling::Flags::BILINEAR,
                )
                .unwrap();

                let mut rgb_frame = ffmpeg::frame::Video::empty();
                scaler.run(&frame, &mut rgb_frame).unwrap();

                sender.send(rgb_frame).unwrap();
                request_redraw();
            }
        }
    }
}
