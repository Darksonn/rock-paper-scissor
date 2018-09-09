use std::sync::mpsc::{Sender, Receiver, channel};
use std::net::{TcpStream, SocketAddr};
use std::io::{Result as IoResult, Error as IoError, Read, Write, ErrorKind};
use std::str::from_utf8;
use rustyline::error::ReadlineError;

extern crate rustyline;
mod listen;

#[derive(Clone,Copy,Debug)]
enum Move {
    Rock, Paper, Scissor,
}
#[derive(Clone,Copy,Debug)]
enum GameOutcome {
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
struct Client {
    pub addr: SocketAddr,
    stream: TcpStream,
    pub name: String,
}
impl Client {
    pub fn new(addr: SocketAddr, mut stream: TcpStream) -> IoResult<Client> {
        let len = {
            let mut len_buf = [0];
            let read = stream.read_exact(&mut len_buf)?;
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
                format!("invalid ping response got {} expected space.", buf[0] as char)))
        }
    }
}

struct State {
    new_clients_send: Sender<(TcpStream, SocketAddr)>,
    new_clients: Receiver<(TcpStream, SocketAddr)>,
    listen_messages_send: Sender<listen::ListenMessage>,
    listen_messages: Receiver<listen::ListenMessage>,
    shutdown_listen: Vec<listen::ShutdownHandle>,
    clients: Vec<Client>,
}
impl State {
    pub fn print_messages(&mut self) {
        while let Ok(msg) = self.listen_messages.try_recv() {
            println!("{}", msg.desc);
            if let Some(io) = msg.err {
                println!("{}", io);
            }
        }
        while let Ok((stream, addr)) = self.new_clients.try_recv() {
            println!("New connection from {}", addr);
            match Client::new(addr, stream) {
                Ok(client) => {
                    println!("Bot {} has name {}", self.clients.len(), client.name);
                    self.clients.push(client);
                },
                Err(err) => {
                    println!("Handshake failed.\n{}", err);
                },
            };
        }
    }
    pub fn ping(&mut self) {
        let mut indexes = Vec::new();
        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.ping() {
                Ok(()) => { },
                Err(err) => {
                    println!("{}\nRemoving client {}.", err, client.name);
                    indexes.push(i);
                },
            }
        }
        for i in indexes.iter().rev().cloned() {
            self.clients.remove(i);
        }
        if self.clients.len() == 0 {
            println!("There are no clients.");
        }
        for (i, client) in self.clients.iter().enumerate() {
            println!("Client {} is called {}.", i, client.name);
        }
    }
    pub fn long_battle(&mut self, bot1: usize, bot2: usize, steps: usize) {
        if self.clients.len() < bot1 {
            println!("no such bot {}", bot1);
        }
        if self.clients.len() < bot2 {
            println!("no such bot {}", bot2);
        }
        if bot1 == bot2 {
            println!("same bot");
        }
        match self.real_long_battle(bot1, bot2, steps) {
            Ok(()) => {
                println!("battle finished");
            },
            Err(err) => {
                println!("battle failed: {}", err);
            },
        }
    }
    fn real_long_battle(
        &mut self,
        bot1: usize,
        bot2: usize,
        steps: usize
    ) -> IoResult<()> {
        self.clients[bot1].new_game()?;
        self.clients[bot2].new_game()?;
        let mut wins1 = 0;
        let mut wins2 = 0;
        let mut ties = 0;
        for i in 0..steps {
            let move1 = self.clients[bot1].get_move()?;
            let move2 = self.clients[bot2].get_move()?;
            println!("moves are {:?} and {:?}", move1, move2);
            match move1.game_outcome(move2) {
                GameOutcome::Win => {
                    wins1 += 1;
                },
                GameOutcome::Lose => {
                    wins2 += 1;
                },
                GameOutcome::Tie => {
                    ties += 1;
                },
            }
            if i == steps-1 {
                self.clients[bot1].end_game(move2)?;
                self.clients[bot2].end_game(move1)?;
            } else {
                self.clients[bot1].cont_game(move2)?;
                self.clients[bot2].cont_game(move1)?;
            }
        }
        println!("{} won {} times.", self.clients[bot1].name, wins1);
        println!("{} won {} times.", self.clients[bot2].name, wins2);
        println!("There were {} ties.", ties);
        Ok(())
    }
    pub fn shutdown(self) {
        for client in self.clients {
            client.shutdown();
        }
        for handle in self.shutdown_listen {
            handle.shutdown();
        }
    }
}

fn main() {
    let (new_clients_send, new_clients) = channel();
    let (listen_messages_send, listen_messages) = channel();
    let listen_shutdown = listen::listen_thread(
        "[::]:4321",
        new_clients_send.clone(),
        listen_messages_send.clone()
    );
    let mut state = State {
        new_clients_send,
        new_clients,
        listen_messages_send,
        listen_messages,
        shutdown_listen: vec![listen_shutdown],
        clients: Vec::new(),
    };
    let rlconfig = rustyline::config::Config::builder()
        .max_history_size(1024)
        .auto_add_history(true)
        .build();

    let mut rl = rustyline::Editor::<()>::with_config(rlconfig);

    loop {
        state.print_messages();
        let cmd_line = match rl.readline(">> ") {
            Ok(cmd) => cmd,
            Err(ReadlineError::Eof) => {
                break;
            },
            Err(ReadlineError::Interrupted) => {
                break;
            },
            err => {
                err.unwrap();
                break;
            }
        };
        let mut chunks = cmd_line.split_whitespace();
        let cmd = match chunks.next() {
            Some(c) => c,
            None => continue,
        };
        if cmd == "exit" {
            break;
        }
        if cmd == "ping" {
            state.ping();
        }
        if cmd == "battle" {
            let bot1: usize = chunks.next().unwrap().parse().unwrap();
            let bot2: usize = chunks.next().unwrap().parse().unwrap();
            let battles: usize = chunks.next().unwrap().parse().unwrap();
            state.long_battle(bot1, bot2, battles);
        }
    }
    state.shutdown();
    println!("goodbye");
}
