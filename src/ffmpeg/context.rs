use ffmpeg_next::ffi::*;
use std::sync::Arc;

pub(crate) struct InputFormat {
    ptr: *mut AVFormatContext,
}

impl InputFormat {
    pub unsafe fn wrap(ptr: *mut AVFormatContext) -> Self {
        InputFormat { ptr }
    }

    pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&self) -> *mut AVFormatContext {
        self.ptr
    }
}

impl Drop for InputFormat {
    fn drop(&mut self) {
        unsafe {
            avformat_close_input(&mut self.ptr);
        }
    }
}

pub(crate) struct InputStream {
    context: Arc<InputFormat>,
    index: usize,
}

impl InputStream {
    pub unsafe fn as_ptr(&self) -> *const AVStream {
        (*self.context.as_ptr()).streams.add(self.index) as *const _
    }
}

impl InputStream {
    pub fn wrap(context: Arc<InputFormat>, index: usize) -> Self {
        InputStream { context, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
