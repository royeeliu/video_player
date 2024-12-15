use crate::*;
use std::sync::{Arc, Mutex};

pub struct DemuxerStream {
    inner: Arc<Mutex<DemuxerStreamInner>>,
}

impl DemuxerStream {
    fn new(inner: Arc<Mutex<DemuxerStreamInner>>) -> Self {
        DemuxerStream { inner }
    }

    pub fn index(&self) -> usize {
        self.inner.lock().unwrap().index()
    }

    pub fn media_type(&self) -> MediaType {
        self.inner.lock().unwrap().media_type()
    }

    pub fn read(&self) -> Option<ffmpeg_next::packet::Packet> {
        unimplemented!()
    }
}

pub struct Demuxer {
    _context: Arc<ffmpeg::context::InputFormat>,
    streams: Vec<Arc<Mutex<DemuxerStreamInner>>>,
}

impl Demuxer {
    pub fn new(source: MediaSource) -> Self {
        let context = Arc::new(source.into_context());
        let nb_streams = unsafe { (*context.as_ptr()).nb_streams as usize };
        let streams = (0..nb_streams)
            .map(|i| Arc::new(Mutex::new(DemuxerStreamInner::new(context.clone(), i))))
            .collect();
        Demuxer {
            _context: context,
            streams,
        }
    }

    pub fn stream(&self, index: usize) -> Option<DemuxerStream> {
        self.streams
            .get(index)
            .map(|s| DemuxerStream::new(s.clone()))
    }
}

struct DemuxerStreamInner {
    context: ffmpeg::context::InputStream,
}

impl DemuxerStreamInner {
    fn new(context: Arc<ffmpeg::context::InputFormat>, index: usize) -> Self {
        DemuxerStreamInner {
            context: ffmpeg::context::InputStream::wrap(context, index),
        }
    }

    fn index(&self) -> usize {
        self.context.index()
    }

    fn media_type(&self) -> MediaType {
        unsafe { ffmpeg::media_type_of_stream(self.context.as_ptr()) }
    }
}
