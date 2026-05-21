use std::{
    io::{Read, Result, Write},
    net::TcpStream,
    sync::{Arc, Mutex, MutexGuard},
};

pub trait ForceLock<T> {
    fn force_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> ForceLock<T> for Mutex<T> {
    fn force_lock(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(i) => i,
            Err(e) => e.into_inner(),
        }
    }
}

#[derive(Clone)]
pub struct SharedStream {
    stream: Arc<TcpStream>,
}

impl SharedStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(stream),
        }
    }
}

impl Read for SharedStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (&*self.stream).read(buf)
    }
}

impl Write for SharedStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (&*self.stream).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (&*self.stream).flush()
    }
}

pub struct RollingAverage<const N: usize> {
    values: [f32; N],
    index: usize,
    full: bool,
}

impl<const N: usize> RollingAverage<N> {
    pub fn new() -> Self {
        Self {
            values: [0.0; N],
            index: 0,
            full: false,
        }
    }

    pub fn push(&mut self, value: f32) {
        self.values[self.index] = value;
        self.index += 1;

        self.full |= self.index == N;
        self.index = self.index % N;
    }

    pub fn avg(&self) -> f32 {
        if self.full {
            self.values.iter().sum::<f32>() / N as f32
        } else {
            self.values.iter().take(self.index).sum::<f32>() / self.index as f32
        }
    }
}

impl<const N: usize> Default for RollingAverage<N> {
    fn default() -> Self {
        Self::new()
    }
}
