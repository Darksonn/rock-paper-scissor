use std::io::{Result as IoResult, Error as IoError, ErrorKind, Read, Write};
use std::net::{ToSocketAddrs, TcpStream};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn next_char(&mut self) -> IoResult<u8> {
        loop {
            let mut buf = [0];
            self.stream.read_exact(&mut buf)?;
            if buf[0] == (' ' as u8) {
                self.stream.write(&[' ' as u8])?;
                self.stream.flush()?;
            } else {
                return Ok(buf[0]);
            }
        }
    }
    pub fn send_byte(&mut self, c: u8) -> IoResult<()> {
        let buf = [c];
        self.stream.write(&buf)?;
        self.stream.flush()?;
        Ok(())
    }
    pub fn connect<A: ToSocketAddrs>(addr: A, name: &str) -> IoResult<Connection> {
        if name.len() > 255 {
            panic!("Name longer than 255 bytes.");
        }
        let mut conn = TcpStream::connect(addr)?;
        let mut buf = [0u8; 258];
        buf[0] = name.len() as u8;
        for (i, b) in name.as_bytes().iter().cloned().enumerate() {
            buf[i+1] = b;
        }
        buf[name.len() + 1] = '\n' as u8;
        conn.write(&buf[0..name.len()+2])?;
        conn.flush()?;
        Ok(Connection {
            stream: conn,
        })
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
