use std::{
    env,
    sync::{mpsc, Arc},
};
use video_renderer::VideoRenderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

extern crate ffmpeg_next as ffmpeg;
type VideoReceiver = mpsc::Receiver<ffmpeg::frame::Video>;

mod presenter;
mod texture;
mod video_renderer;
mod wgpu_context;

pub struct App {
    window: Option<Arc<Window>>,
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
            let attr = Window::default_attributes().with_title("VideoPlayer");
            let window = event_loop.create_window(attr).unwrap();
            let window = Arc::new(window);
            self.renderer.init(window.clone());
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
    env_logger::init();
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
        if packet.read(&mut ictx).is_err() {
            break;
        }
        if packet.stream() == video_stream_index {
            packet_sender.send(packet).unwrap();
        }
    });

    std::thread::spawn(move || loop {
        let mut frame = ffmpeg::frame::Video::empty();
        while unsafe { frame.is_empty() } {
            if let Some(packet) = packet_receiver.recv().ok() {
                decoder.send_packet(&packet).unwrap();
                decoder.receive_frame(&mut frame).err();
            } else {
                break;
            }
        }
        if unsafe { frame.is_empty() } {
            break;
        }
        video_sender.send(frame).unwrap();
    });

    let mut app = App::new(video_receiver);

    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).expect("Run app failed");
}
