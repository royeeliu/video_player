pub mod demuxer;
pub mod video_renderer;

mod convert_from_yuv;
mod presenter;
mod texture;
mod wgpu_context;

pub enum MediaType {
    Video,
    Audio,
    Data,
    Subtitle,
    Attachment,
}

pub fn init() {
    ffmpeg_next::init().unwrap();
}
