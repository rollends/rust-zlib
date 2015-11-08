use zlib_thread::ZlibCoprocessorCommand;
use ZlibProxy;

/// Zlib Proxy can be cloned to allow other threads to communicate with the Zlib Worker Thread.
impl Clone for ZlibProxy {
    fn clone(&self) -> ZlibProxy {
        {
            // Increase Proxy Count
            let mut count = self.proxy_count.lock().unwrap();
            *count += 1;
        }
        ZlibProxy {
            job_channel : self.job_channel.clone(),
            proxy_count : self.proxy_count.clone(),
        }
    }
}

/// Drop is used to reference count the number of proxies referring to a given Zlib Worker Thread.
/// Once the last proxy is dropped, the Zlib Worker Thread is sent a Quit message and it should exit.
impl Drop for ZlibProxy {
    fn drop(& mut self) {
        let mut count = self.proxy_count.lock().unwrap();
        *count -= 1;
        if *count == 0 {
            self.job_channel.send( ZlibCoprocessorCommand::Quit ).unwrap();
        }
    }
}
