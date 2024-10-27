use softbuffer::Context;
use std::{env, rc::Rc, sync::mpsc};
use video_renderer::VideoRenderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

extern crate ffmpeg_next as ffmpeg;
type VideoReceiver = mpsc::Receiver<ffmpeg::frame::Video>;

mod video_renderer;

pub struct App {
    window: Option<Rc<Window>>,
    renderer: VideoRenderer,
}

impl App {
    pub fn new(video_receiver: VideoReceiver) -> Self {
        App {
            window: None,
            renderer: VideoRenderer::new(video_receiver),
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
            self.renderer.init(&context, window.clone());
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(window) = self.window.as_ref() {
                    self.renderer.resize(size);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.renderer.render();
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
