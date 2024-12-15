use crate::NONE_CODEC_PARAMETERS;
use ffmpeg_next::ffi::AVStream;

pub mod context;

pub unsafe fn media_type_of_stream(stream: *const AVStream) -> crate::MediaType {
    (*stream)
        .codecpar
        .as_ref()
        .map(|codecpar| codecpar.codec_type.into())
        .unwrap_or(crate::MediaType::Unknown(NONE_CODEC_PARAMETERS))
}
