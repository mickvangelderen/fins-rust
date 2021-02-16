struct Connection {
    stream: TcpStream,
    buffer: Box<[u8; 4096]>,
    offset: usize,
    filled: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) {
        Self {
            stream,
            buffer: Box::new([0; 4096]),
            offset: 0,
            filled: 0,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            self.make_room();

            match self.stream.read(&mut self.buffer[self.filled..]).await {
                Ok(0) => return Ok(None),
                Ok(n) => {
                    self.filled += n;
                },
                Err(e) => return Err(e.into())
            }
        }
    }

    fn make_room(&mut self) {
        if self.filled > self.offset {
            self.buffer.copy_within(self.filled..self.offset, 0);
            self.filled -= self.offset;
            self.offset = 0;
        }
    }
}

pub type Result<T> = Result<T, Error>;

pub enum Error {
    Io(std::io::Error)
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct Frame {}
