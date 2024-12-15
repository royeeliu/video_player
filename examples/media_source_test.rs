use std::env;
use vtk::*;

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
    let media_source = MediaSource::open(path);
    let format = media_source.format();
    println!("Format: {}", format.name());

    media_source.all_streams().iter().for_each(|stream| {
        println!("Stream {}: {:?}", stream.index(), stream.media_type());
    });
}
