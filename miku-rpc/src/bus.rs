#[cfg(feature = "wrappers")]
use crate::wrappers::IdentifiedDevice;
use crate::{types::DeviceList, Call, Response};
use epoll_rs::{Epoll, Opts as PollOpts};
use miniserde_miku::{json, Deserialize, Serialize};

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use std::str;
use std::time::Duration;
use termios::*;

use arrayvec::ArrayString;

/// A bus interface to the HLApi
pub struct DeviceBus {
    file: File,
    buffer: [u8; 4096],
    write_buffer: ArrayString<4096>,
    string_buf: String,
    poller: Epoll,
}

impl DeviceBus {
    pub fn new(path: impl AsRef<Path>) -> io::Result<DeviceBus> {
        let inner_f = File::options().read(true).write(true).open(path)?;

        let poller = Epoll::new()?;
        let inner_f = poller.add(inner_f, PollOpts::IN)?.into_file();

        let mut termios = Termios::from_fd(inner_f.as_raw_fd())?;
        cfmakeraw(&mut termios);
        termios.c_lflag &= !ECHO;
        tcsetattr(inner_f.as_raw_fd(), TCSANOW, &termios)?;

        Ok(DeviceBus {
            file: inner_f,
            buffer: [0; 4096],
            string_buf: String::with_capacity(2048),
            write_buffer: ArrayString::<4096>::new(),
            poller,
        })
    }

    /// Calls a HLApi method and gets its response.
    pub fn call<T: Serialize, R: Deserialize>(&mut self, msg: &Call<T>) -> io::Result<Response<R>> {
        self.flush()?;
        self.write_message(msg)?;
        self.read_message()
    }

    /// Calls a HLApi method and gets its response. Uses a pre-serialized string to help with optimizations for zero-argument functions.
    pub fn call_preserialized<R: Deserialize>(&mut self, msg: &[u8]) -> io::Result<Response<R>> {
        self.flush()?;
        self.file.write_all(msg)?;
        self.read_message()
    }

    /// Utility method to create a wrapper for a device of a certain type.
    #[cfg(feature = "wrappers")]
    pub fn wrap<T: IdentifiedDevice>(&mut self) -> io::Result<Option<T>> {
        Ok(self.find(T::IDENTITY)?.map(T::from_id))
    }

    /// Utility method to find a device id for a certain device type.
    pub fn find(&mut self, kind: &str) -> io::Result<Option<String>> {
        let device_list: DeviceList = self.call(&Call::list())?;
        Ok(device_list
            .data
            .into_iter()
            .find(|v| v.type_names.iter().any(|s| s == kind))
            .map(|v| v.device_id))
    }

    fn write_message<T: Serialize>(&mut self, msg: &Call<T>) -> io::Result<()> {
        self.write_buffer.clear();
        self.write_buffer.push('\0');
        let start = std::time::Instant::now();
        json::to_string::<_, 4096, 16384>(msg, &mut self.write_buffer);
        println!("{:?}", start.elapsed());
        self.write_buffer.push('\0');

        self.file.write_all(self.write_buffer.as_bytes())?;
        Ok(())
    }

    fn read_message<R: Deserialize>(&mut self) -> io::Result<Response<R>> {
        let mut bytes_read = self.read()?;

        if unsafe { *self.buffer.get_unchecked(bytes_read - 1) } != 0u8 {
            self.string_buf
                .push_str(unsafe { str::from_utf8_unchecked(&self.buffer[1..bytes_read]) });

            loop {
                bytes_read = self.read()?;

                if unsafe { *self.buffer.get_unchecked(bytes_read - 1) } == 0u8 {
                    self.string_buf.push_str(unsafe {
                        str::from_utf8_unchecked(&self.buffer[..bytes_read - 1])
                    });
                    break;
                } else {
                    self.string_buf
                        .push_str(unsafe { str::from_utf8_unchecked(&self.buffer[..bytes_read]) });
                }
            }
        } else {
            self.string_buf
                .push_str(unsafe { str::from_utf8_unchecked(&self.buffer[1..bytes_read - 1]) });
        }

        json::from_str(&self.string_buf).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.string_buf.clear();

        while self
            .poller
            .wait_one_timeout(Duration::from_secs(0))?
            .is_some()
        {
            self.file.read(&mut self.buffer)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn read(&mut self) -> io::Result<usize> {
        self.poller.wait_one()?;
        self.file.read(&mut self.buffer)
    }
}
