use std::{
    path::Path,
    sync::{mpsc, Arc},
};

use crate::MediaType;

extern crate ffmpeg_next as ffmpeg;
type PacketReceiver = mpsc::Receiver<ffmpeg::packet::Packet>;
type PacketSender = mpsc::SyncSender<ffmpeg::packet::Packet>;

pub struct DemuxerStream {
    packet_receiver: PacketReceiver,
}

impl DemuxerStream {
    pub fn new(packet_receiver: PacketReceiver) -> Self {
        DemuxerStream { packet_receiver }
    }

    pub fn index(&self) -> usize {
        unimplemented!()
    }

    fn is_enabled(&self) -> bool {
        unimplemented!()
    }

    pub fn enable(&mut self) {
        unimplemented!()
    }

    pub fn disable(&mut self) {
        unimplemented!()
    }

    pub fn read(&self) -> Option<ffmpeg::packet::Packet> {
        self.packet_receiver.try_recv().ok()
    }
}

pub struct Demuxer {
    ictx: ffmpeg::format::context::Input,
}

impl Demuxer {
    pub fn new(path: &Path) -> Self {
        let mut ictx = ffmpeg::format::input(path).unwrap();
        Demuxer { ictx }
    }

    pub fn format(&self) -> String {
        self.ictx.format().name().to_string()
    }

    pub fn all_streams(&self) -> Vec<Arc<DemuxerStream>> {
        vec![]
    }

    pub fn get_stream(&self, index: usize) -> Option<Arc<DemuxerStream>> {
        None
    }

    pub fn get_best_stream(&self, media_type: MediaType) -> Option<Arc<DemuxerStream>> {
        None
    }

    pub fn start(&self) {
        unimplemented!()
    }
}
