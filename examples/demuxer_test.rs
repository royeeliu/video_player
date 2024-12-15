use std::env;

use vtk::{demuxer::Demuxer, MediaType};

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

    vtk::init();
    let demuxer = Demuxer::new(path);
    println!("Format: {}", demuxer.format());

    if let Some(stream) = demuxer.get_best_stream(MediaType::Video) {
        println!("Stream index: {}", stream.index());
    }
}
