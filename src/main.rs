use softbuffer::{Context, Surface};
use std::{env, num::NonZeroU32, rc::Rc, sync::mpsc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

extern crate ffmpeg_next as ffmpeg;
use ffmpeg::software::scaling;

type VideoReceiver = mpsc::Receiver<ffmpeg::frame::Video>;

pub struct App {
    video_receiver: VideoReceiver,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl App {
    pub fn new(video_receiver: VideoReceiver) -> Self {
        App {
            video_receiver,
            window: None,
            surface: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resumed");
        if self.window.is_none() {
            let attr = Window::default_attributes().with_title("Soft Render Example");
            let window = event_loop.create_window(attr).unwrap();
            let window = Rc::new(window);
            let context = Context::new(window.clone()).unwrap();
            let surface = Surface::new(&context, window.clone()).unwrap();
            self.window = Some(window);
            self.surface = Some(surface);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                if let (Some(width), Some(height)) = {
                    let size = self.window.as_ref().unwrap().inner_size();
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                } {
                    if let Some(surface) = self.surface.as_mut() {
                        surface.resize(width, height).unwrap();

                        let yuv_frame = self.video_receiver.recv().unwrap();

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
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::CloseRequested => {
                println!("Close requested");
                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn main() {
    if env::args().len() < 2 {
        println!("ERROR: No input file.");
        return;
    }

    let file_name = env::args().nth(1).unwrap();
    println!("Input file: {}", file_name);

    let path = std::path::Path::new(&file_name);
    if !path.exists() {
        println!("ERROR: File not found.");
        return;
    }

    ffmpeg::init().unwrap();

    let mut ictx = ffmpeg::format::input(path).unwrap();
    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)
        .unwrap();

    let video_stream_index = input.index();

    let context_decoder: ffmpeg::codec::Context =
        ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
    let mut decoder = context_decoder.decoder().video().unwrap();
    println!(
        "\nVideo: {:?}, {}x{}",
        decoder.id(),
        decoder.width(),
        decoder.height()
    );

    let (packet_sender, packet_receiver) = mpsc::sync_channel(10);
    let (video_sender, video_receiver) = mpsc::sync_channel(1);

    std::thread::spawn(move || loop {
        let mut packet = ffmpeg::codec::packet::Packet::empty();
        packet.read(&mut ictx).unwrap();
        if packet.stream() == video_stream_index {
            packet_sender.send(packet).unwrap();
        }
    });

    std::thread::spawn(move || loop {
        let mut frame = ffmpeg::frame::Video::empty();
        while unsafe { frame.is_empty() } {
            let packet = packet_receiver.recv().unwrap();
            decoder.send_packet(&packet).unwrap();
            decoder.receive_frame(&mut frame).err();
        }
        video_sender.send(frame).unwrap();
    });

    let mut app = App::new(video_receiver);

    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).expect("Run app failed");
}
