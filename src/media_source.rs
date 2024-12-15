use crate::*;
use std::{
    ffi::{c_int, CStr, CString},
    path::Path,
    ptr,
    str::from_utf8_unchecked,
};

pub struct Format {
    ptr: *const AVInputFormat,
}

impl Format {
    pub(crate) fn wrap(ptr: *const AVInputFormat) -> Self {
        Format { ptr }
    }

    pub fn name(&self) -> &str {
        if self.ptr.is_null() {
            ""
        } else {
            unsafe { from_utf8_unchecked(CStr::from_ptr((*self.ptr).name).to_bytes()) }
        }
    }
}

pub struct StreamInfo {
    index: usize,
    media_type: MediaType,
}

impl StreamInfo {
    pub(crate) fn new(index: usize, media_type: MediaType) -> Self {
        StreamInfo { index, media_type }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn media_type(&self) -> MediaType {
        self.media_type
    }
}

pub struct MediaSource {
    context: ffmpeg::context::InputFormat,
}

impl MediaSource {
    pub fn open(path: &Path) -> Self {
        let path = CString::new(path.as_os_str().to_str().unwrap()).unwrap();
        unsafe {
            let mut context = ptr::null_mut();
            match avformat_open_input(
                &mut context,
                path.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            ) {
                0 => (),
                e => panic!("Error: {}", e),
            }

            let context = ffmpeg::context::InputFormat::wrap(context);
            match avformat_find_stream_info(context.as_mut_ptr(), ptr::null_mut()) {
                r if r >= 0 => MediaSource { context },
                e => panic!("Error: {}", e),
            }
        }
    }

    pub fn format(&self) -> Format {
        unsafe { Format::wrap((*self.context.as_ptr()).iformat as *mut AVInputFormat) }
    }

    pub fn all_streams(&self) -> Vec<StreamInfo> {
        unsafe {
            let nb_streams = (*self.context.as_ptr()).nb_streams as usize;
            (0..nb_streams)
                .map(|i| {
                    let stream = *(*self.context.as_ptr()).streams.add(i);
                    StreamInfo::new(i, ffmpeg::media_type_of_stream(stream))
                })
                .collect()
        }
    }

    pub fn stream(&self, index: usize) -> Option<StreamInfo> {
        unsafe {
            let nb_streams = (*self.context.as_ptr()).nb_streams as usize;
            if index >= nb_streams {
                return None;
            }
            let stream = *(*self.context.as_ptr()).streams.add(index);
            Some(StreamInfo::new(index, ffmpeg::media_type_of_stream(stream)))
        }
    }

    pub fn best_stream(&self, media_type: MediaType) -> Option<StreamInfo> {
        unsafe {
            let decoder = ptr::null_mut();
            let index = av_find_best_stream(
                self.context.as_mut_ptr(),
                media_type.into(),
                -1 as c_int,
                -1 as c_int,
                decoder,
                0,
            );
            self.stream(index as usize)
        }
    }

    pub(crate) fn into_context(self) -> ffmpeg::context::InputFormat {
        self.context
    }
}
