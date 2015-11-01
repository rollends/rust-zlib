use zlib_thread::ZlibCoprocessorCommand;
use ZlibProxy;

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

impl Drop for ZlibProxy {
    fn drop(& mut self) {
        let mut count = self.proxy_count.lock().unwrap();
        *count -= 1;
        if *count == 0 {
            self.job_channel.send( ZlibCoprocessorCommand::Quit ).unwrap();
        }
    }
}
