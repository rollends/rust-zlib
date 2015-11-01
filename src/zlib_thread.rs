extern crate libc;

use std::sync::mpsc::{channel,Sender, Receiver};
use std::thread;
use ZlibEvent;

#[link(name = "z")]
extern {
    fn compress2 (
        dest : * mut libc::c_uchar,
        destLen : * mut libc::c_ulong,
        source : * const libc::c_uchar,
        sourceLen : libc::c_ulong,
        level : libc::c_int,
    ) -> i32;
    fn uncompress (
        dest : * mut libc::c_uchar,
        destLen : * mut libc::c_ulong,
        source : * const libc::c_uchar,
        sourceLen : libc::c_ulong,
    ) -> i32;
    fn compressBound ( sourceLen : libc::c_ulong ) -> libc::c_ulong;
}

/// Possible commands to be sent to the Zlib Coprocessor (worker thread). Shouldn't be used
/// directly, although it is technically possible (and would work!).
pub enum ZlibCoprocessorCommand {
    Compress( Vec<u8>, Sender<ZlibEvent> ),
    Uncompress( Vec<u8>, usize, Sender<ZlibEvent> ),
    Quit,
}

/// Zlib Worker Thread
/// It listens for commands via a `Receiver` channel.
pub struct ZlibCoprocessor {
    rx : Receiver<ZlibCoprocessorCommand>,
}

impl ZlibCoprocessor {
    /// Constructs a Zlib Coprocessor within a newly spawned thread and executes the main message loop
    /// within the new thread. Returns the `Sender` channel used to enqueue jobs for the coprocessor.
    pub fn init() -> Sender<ZlibCoprocessorCommand> {
        let (s,r) = channel();
        thread::spawn( move || {
            let mut zlib = ZlibCoprocessor{ rx : r };
            zlib.main();
        });
        s
    }

    unsafe fn uncompress( &self, buffer : Vec<u8>, output_size : usize, signal : Sender<ZlibEvent> ) {
        let src_len = buffer.len() as libc::c_ulong;
        let mut len = output_size as libc::c_ulong;
        let mut array = Vec::<u8>::with_capacity(output_size);
        let code =  uncompress(
                        array.as_mut_ptr(),
                        & mut len as * mut libc::c_ulong,
                        buffer.as_ptr(),
                        src_len
                    );

        if len == 0 || code < 0 {
            signal.send( ZlibEvent::UncompressFailed ).ok();
            return;
        }

        array.set_len( len as usize );
        signal.send( ZlibEvent::UncompressCompleted( array ) ).ok();
    }

    unsafe fn compress( &self, buffer : Vec<u8>, signal : Sender<ZlibEvent> ) {
        let src_len = buffer.len() as libc::c_ulong;
        let mut len = compressBound( src_len ) as libc::c_ulong;
        let mut array = Vec::<u8>::with_capacity(len as usize);
        let code =  compress2(
                        array.as_mut_ptr(),
                        & mut len as * mut libc::c_ulong,
                        buffer.as_ptr(),
                        src_len,
                        9
                    );

        if len == 0 || code < 0 {
            signal.send( ZlibEvent::CompressFailed ).ok();
            return;
        }

        array.set_len( len as usize );
        signal.send( ZlibEvent::CompressCompleted( array ) ).ok();
    }

    /// Main Message Loop for Zlib Coprocessor.
    fn main( & mut self ) {
        loop {
            match self.rx.recv().unwrap() {
                ZlibCoprocessorCommand::Quit => break,
                ZlibCoprocessorCommand::Compress( buffer, signal ) => {
                    unsafe {
                        self.compress( buffer, signal );
                    }
                },
                ZlibCoprocessorCommand::Uncompress( buffer, output_size, signal ) => {
                    unsafe {
                        self.uncompress( buffer, output_size, signal );
                    }
                }
            }
        }
    }
}
