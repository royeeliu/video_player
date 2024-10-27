use softbuffer::{Context, Surface};
use std::{num::NonZeroU32, rc::Rc, sync::mpsc};
use winit::{dpi::PhysicalSize, window::Window};

extern crate ffmpeg_next as ffmpeg;
use ffmpeg::software::scaling;
type VideoReceiver = mpsc::Receiver<ffmpeg::frame::Video>;

pub struct VideoRenderer {
    video_receiver: VideoReceiver,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl VideoRenderer {
    pub fn new(video_receiver: VideoReceiver) -> Self {
        VideoRenderer {
            video_receiver,
            surface: None,
        }
    }

    pub fn init(&mut self, context: &Context<Rc<Window>>, window: Rc<Window>) {
        self.surface = Some(Surface::new(context, window).unwrap());
    }

    pub fn render(&mut self) {
        if let Some(surface) = self.surface.as_mut() {
            let size = surface.window().inner_size();
            if let (Some(width), Some(height)) =
                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
            {
                surface.resize(width, height).unwrap();
                let yuv_frame = self.video_receiver.recv().unwrap();
                Self::render_frame(surface, width, height, yuv_frame);
            }
        }
    }

    pub fn resize(&mut self, _size: PhysicalSize<u32>) {}

    fn render_frame(
        surface: &mut Surface<Rc<Window>, Rc<Window>>,
        width: NonZeroU32,
        height: NonZeroU32,
        yuv_frame: ffmpeg::frame::Video,
    ) {
        let mut scaler = scaling::Context::get(
            yuv_frame.format(),
            yuv_frame.width(),
            yuv_frame.height(),
            ffmpeg::util::format::Pixel::RGBA,
            width.get(),
            height.get(),
            scaling::Flags::BILINEAR,
        )
        .unwrap();

        let mut rgb_frame = ffmpeg::frame::Video::empty();
        scaler.run(&yuv_frame, &mut rgb_frame).unwrap();

        let mut buffer = surface.buffer_mut().unwrap();
        let data = rgb_frame.data(0);
        for y in 0..height.get() {
            for x in 0..width.get() {
                let index = (y * rgb_frame.stride(0) as u32 + x * 4) as usize;
                let pixel = &data[index..index + 3];
                let red = pixel[0] as u32;
                let green = pixel[1] as u32;
                let blue = pixel[2] as u32;
                let index = y as usize * width.get() as usize + x as usize;
                buffer[index] = blue | (green << 8) | (red << 16);
            }
        }
        buffer.present().unwrap();
    }
}
