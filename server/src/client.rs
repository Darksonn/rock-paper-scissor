use std::net::{TcpStream, SocketAddr};
use std::io::{Result as IoResult, Error as IoError, Read, Write, ErrorKind};
use std::str::from_utf8;
use std::time::Duration;

pub struct Client {
    pub addr: SocketAddr,
    stream: TcpStream,
    pub name: String,
}
impl Client {
    pub fn new(addr: SocketAddr, mut stream: TcpStream) -> IoResult<Client> {
        stream.set_read_timeout(Some(Duration::new(10, 0)))?;
        stream.set_write_timeout(Some(Duration::new(10, 0)))?;
        let len = {
            let mut len_buf = [0];
            stream.read_exact(&mut len_buf)?;
            usize::from(len_buf[0])
        };
        let name = {
            let mut name_buf = [0; 257];
            stream.read_exact(&mut name_buf[0..len+1])?;
            match from_utf8(&name_buf[0..len]) {
                Ok(s) => String::from(s),
                Err(_) => return Err(IoError::new(ErrorKind::InvalidData,
                                                  "name invalid utf8")),
            }
        };
        Ok(Client {
            addr,
            stream,
            name,
        })
    }
    pub fn set_timeout(&mut self, secs: u64) -> IoResult<()> {
        self.stream.set_read_timeout(Some(Duration::new(secs, 0)))?;
        self.stream.set_write_timeout(Some(Duration::new(secs, 0)))?;
        Ok(())
    }
    pub fn remove_timeout(&mut self) -> IoResult<()> {
        self.stream.set_read_timeout(None)?;
        self.stream.set_write_timeout(None)?;
        Ok(())
    }
    pub fn shutdown(mut self) {
        let _ = self.stream.write(&['x' as u8]);
        let _ = self.stream.flush();
    }
    pub fn new_game(&mut self) -> IoResult<()> {
        self.stream.write(&['n' as u8])?;
        self.stream.flush()?;
        Ok(())
    }
    pub fn cont_game(&mut self, m: Move) -> IoResult<()> {
        self.stream.write(&[m.into_u8()])?;
        self.stream.flush()?;
        Ok(())
    }
    pub fn end_game(&mut self, m: Move) -> IoResult<()> {
        self.stream.write(&[m.into_u8_end()])?;
        self.stream.flush()?;
        Ok(())
    }
    pub fn destroy_game(&mut self) -> IoResult<()> {
        self.stream.write(&['e' as u8])?;
        self.stream.flush()?;
        self.stream.set_nonblocking(true)?;
        let mut buf = [0; 1024];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        self.stream.set_nonblocking(false)?;
        Ok(())
    }
    pub fn get_move(&mut self) -> IoResult<Move> {
        let mut buf = [0];
        self.stream.read_exact(&mut buf)?;
        Move::try_from(buf[0])
    }
    pub fn ping(&mut self) -> IoResult<()> {
        self.stream.write(&[' ' as u8])?;
        self.stream.flush()?;
        let mut buf = [0];
        self.stream.read_exact(&mut buf)?;
        if buf[0] == (' ' as u8) {
            Ok(())
        } else {
            Err(IoError::new(ErrorKind::InvalidData,
                format!("invalid ping response got {} expected space.", buf[0] as u8)))
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub enum Move {
    Rock, Paper, Scissor,
}
#[derive(Clone,Copy,Debug)]
pub enum GameOutcome {
    Win, Lose, Tie,
}
impl Move {
    pub fn try_from(byte: u8) -> IoResult<Move> {
        Ok(match byte as char {
            'r' => {
                Move::Rock
            },
            'p' => {
                Move::Paper
            },
            's' => {
                Move::Scissor
            },
            _ => {
                return Err(IoError::new(ErrorKind::InvalidData,
                    format!("byte {} is not r, p or s", byte)));
            }
        })
    }
    /// Returns win if self wins.
    pub fn game_outcome(self, other: Move) -> GameOutcome {
        match self {
            Move::Rock => {
                match other {
                    Move::Rock => GameOutcome::Tie,
                    Move::Paper => GameOutcome::Lose,
                    Move::Scissor => GameOutcome::Win,
                }
            },
            Move::Paper => {
                match other {
                    Move::Rock => GameOutcome::Win,
                    Move::Paper => GameOutcome::Tie,
                    Move::Scissor => GameOutcome::Lose,
                }
            },
            Move::Scissor => {
                match other {
                    Move::Rock => GameOutcome::Lose,
                    Move::Paper => GameOutcome::Win,
                    Move::Scissor => GameOutcome::Tie,
                }
            },
        }
    }
    pub fn into_u8(self) -> u8 {
        (match self {
            Move::Rock => 'r',
            Move::Paper => 'p',
            Move::Scissor => 's',
        }) as u8
    }
    pub fn into_u8_end(self) -> u8 {
        self.into_u8() ^ (' ' as u8)
    }
}
