use app::App;
use winit::event_loop::EventLoop;

extern crate ffmpeg_next as ffmpeg;
mod app;

fn main() {
    ffmpeg::init().unwrap();

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Run app failed");
}
