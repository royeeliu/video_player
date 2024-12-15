use ffmpeg_next::ffi::AVMediaType::*;
use ffmpeg_next::ffi::*;

pub mod demuxer;
pub mod media_source;
pub mod video_renderer;
pub use self::media_source::*;

mod convert_from_yuv;
mod ffmpeg;
mod presenter;
mod texture;
mod wgpu_context;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum MediaType {
    Unknown(i32),
    Video,
    Audio,
    Data,
    Subtitle,
    Attachment,
}

const NONE_CODEC_PARAMETERS: i32 = -100;

impl From<AVMediaType> for MediaType {
    fn from(value: AVMediaType) -> Self {
        match value as i32 {
            v if v < AVMEDIA_TYPE_UNKNOWN as i32 || v > AVMEDIA_TYPE_NB as i32 => {
                MediaType::Unknown(v)
            }
            v => match value {
                AVMEDIA_TYPE_UNKNOWN => MediaType::Unknown(v),
                AVMEDIA_TYPE_VIDEO => MediaType::Video,
                AVMEDIA_TYPE_AUDIO => MediaType::Audio,
                AVMEDIA_TYPE_DATA => MediaType::Data,
                AVMEDIA_TYPE_SUBTITLE => MediaType::Subtitle,
                AVMEDIA_TYPE_ATTACHMENT => MediaType::Attachment,
                AVMEDIA_TYPE_NB => MediaType::Unknown(v),
            },
        }
    }
}

impl From<MediaType> for AVMediaType {
    fn from(value: MediaType) -> Self {
        match value {
            MediaType::Unknown(_) => AVMEDIA_TYPE_UNKNOWN,
            MediaType::Video => AVMEDIA_TYPE_VIDEO,
            MediaType::Audio => AVMEDIA_TYPE_AUDIO,
            MediaType::Data => AVMEDIA_TYPE_DATA,
            MediaType::Subtitle => AVMEDIA_TYPE_SUBTITLE,
            MediaType::Attachment => AVMEDIA_TYPE_ATTACHMENT,
        }
    }
}

pub fn init() {
    ffmpeg_next::init().unwrap();
}
