#![feature(libc)]

mod zlib_proxy;
mod zlib_thread;

pub use std::sync::mpsc::Receiver;

use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

/// Proxy that provides an interface to communicate with the Zlib Worker Thread
pub struct ZlibProxy {
    job_channel : Sender<zlib_thread::ZlibCoprocessorCommand>,
    proxy_count : Arc< Mutex< u8 > >,
}

/// Events produced by the Zlib Worker Thread
pub enum ZlibEvent {
    /// Compression Successful; result is in `Vec<u8>`
    CompressCompleted( Vec<u8> ),
    /// Uncompression Successful; result is in `Vec<u8>`
    UncompressCompleted( Vec<u8> ),
    /// Failed to compress for arbitrary reason.
    CompressFailed,
    /// Failed to uncompress for arbitrary reason.
    UncompressFailed,
}

impl ZlibProxy {

    /// Construct a new Zlib Proxy
    /// The creation of the proxy also instantiates a new Zlib Worker thread in the background.
    /// This means, usually, that `new` should be called only once and then new instances should be
    /// created using `clone`.
    pub fn new() -> ZlibProxy {
        ZlibProxy {
            job_channel : zlib_thread::ZlibCoprocessor::init(),
            proxy_count : Arc::new( Mutex::new(1) ),
        }
    }

    /// Enqueues a compress job. Returns a receiver that can be used to listen for the result event.
    pub fn compress( &self, input : Vec<u8> ) -> Receiver<ZlibEvent> {
        let (s,r) = channel();
        self.job_channel.send( zlib_thread::ZlibCoprocessorCommand::Compress(input, s) ).unwrap();
        r
    }

    /// Enqueues a uncompress job. Returns a receiver that can be used to listen for the result event.
    pub fn uncompress( &self, input : Vec<u8>, output_size : usize ) -> Receiver<ZlibEvent> {
        let (s,r) = channel();
        self.job_channel.send( zlib_thread::ZlibCoprocessorCommand::Uncompress(input, output_size, s) ).unwrap();
        r
    }
}
