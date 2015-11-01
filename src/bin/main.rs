#![feature(convert)]

extern crate zlib;

use std::fs::File;
use std::io::{Read,Write};

fn main() {
    let z = zlib::ZlibProxy::new();
    let full_size;
    {
        let mut f = File::open("/home/rs2dsouz/Documents/adas/Videos/Edges.avi").unwrap();
        let mut fo = File::create("/home/rs2dsouz/Documents/adas/Videos/Compressed_Edges.z").unwrap();

        let mut buffer = Vec::new();
        f.read_to_end(& mut buffer).unwrap();
        full_size = buffer.len();

        match z.compress(buffer).recv().unwrap() {
            zlib::ZlibEvent::CompressCompleted( v ) => fo.write_all(v.as_slice()).unwrap(),
            _ => println!("Failed."),
        };
    }
    {
        let mut f = File::open("/home/rs2dsouz/Documents/adas/Videos/Compressed_Edges.z").unwrap();
        let mut fo = File::create("/home/rs2dsouz/Documents/adas/Videos/Uncompressed_Edges.avi").unwrap();

        let mut buffer = Vec::new();
        f.read_to_end(& mut buffer).unwrap();

        match z.uncompress(buffer, full_size).recv().unwrap() {
            zlib::ZlibEvent::UncompressCompleted( v ) => fo.write_all(v.as_slice()).unwrap(),
            _ => println!("Failed."),
        };
    }
}
