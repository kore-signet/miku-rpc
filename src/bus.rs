use crate::{types::DeviceList, Call, Response};
use epoll_rs::{Epoll, Opts as PollOpts};
use miniserde::{json, Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use std::str;
use std::time::Duration;
use termios::*;

pub struct DeviceBus {
    file: File,
    buffer: VecDeque<u8>,
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
            buffer: VecDeque::with_capacity(1024),
            poller,
        })
    }

    pub fn call<T: Serialize, R: Deserialize>(&mut self, msg: &Call<T>) -> io::Result<Response<R>> {
        self.flush()?;
        self.write_message(msg)?;
        self.read_message()
    }

    /// Utility method to find a device id for a certain device type.
    pub fn find(&mut self, kind: impl Into<String>) -> io::Result<Option<String>> {
        let device_list: DeviceList = self.call(&Call::list())?;
        let kind = kind.into();
        Ok(device_list
            .data
            .into_iter()
            .find(|v| v.type_names.contains(&kind))
            .map(|v| v.device_id))
    }

    fn write_message<T: Serialize>(&mut self, msg: &Call<T>) -> io::Result<()> {
        self.file
            .write_all(format!("\0{}\0", json::to_string(msg)).as_bytes())?;
        Ok(())
    }

    fn read_message<R: Deserialize>(&mut self) -> io::Result<Response<R>> {
        let mut res: Vec<u8> = Vec::new();
        self.read_one()?; // discard initial null byte

        loop {
            let next = self.read_one()?;
            if next == 0 {
                break;
            }

            res.push(next);
        }

        str::from_utf8(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .and_then(|v| {
                json::from_str(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.clear();
        let mut pain: [u8; 1] = [0; 1];
        while self
            .poller
            .wait_one_timeout(Duration::from_secs(0))?
            .is_some()
        {
            self.file.read_exact(&mut pain)?;
        }

        Ok(())
    }

    fn fill_buffer(&mut self) -> io::Result<()> {
        self.poller.wait_one()?;
        self.buffer.clear();

        // lol. lmao
        let mut pain: [u8; 1] = [0; 1];

        while self.buffer.len() < 1024
            && self
                .poller
                .wait_one_timeout(Duration::from_secs(0))?
                .is_some()
        {
            self.file.read_exact(&mut pain)?;
            self.buffer.push_back(pain[0]);
        }

        Ok(())
    }

    fn read_one(&mut self) -> io::Result<u8> {
        if self.buffer.is_empty() {
            self.fill_buffer()?;
        }

        self.buffer.pop_front().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected end of read buffer for tty - maybe an error with the method invoked?",
            )
        })
    }
}
