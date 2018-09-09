use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::{Result as IoResult};
use rustyline::error::ReadlineError;
use client::*;
use statrs::function::erf::erf;

extern crate statrs;
extern crate rustyline;
mod listen;
mod client;

#[allow(dead_code)]
struct State {
    new_clients_send: Sender<Client>,
    new_clients: Receiver<Client>,
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
        while let Ok(client) = self.new_clients.try_recv() {
            self.clients.push(client);
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
    pub fn set_timeout(&mut self, timeout: Option<u64>) {
        let mut indexes = Vec::new();
        for (i, client) in self.clients.iter_mut().enumerate() {
            let client_res = match timeout {
                Some(t) => client.set_timeout(t),
                None => client.remove_timeout(),
            };
            match client_res {
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
                let _ = self.clients[bot1].destroy_game();
                let _ = self.clients[bot2].destroy_game();
            },
        }
    }
    fn real_long_battle(
        &mut self,
        bot1: usize,
        bot2: usize,
        steps: usize
    ) -> IoResult<()> {
        use std::time::Instant;
        let now = Instant::now();
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

        let diff = (wins1 - wins2) as f64;
        let mean = 0f64;
        let stddev_times_sqrt2 = ((4*steps) as f64 / 3f64).sqrt();
        let cdf1 = 0.5 * (1. + erf((diff - mean)/(stddev_times_sqrt2)));
        let cdf2 = 0.5 * (1. + erf((mean - diff)/(stddev_times_sqrt2)));

        println!("{} won {} times.", self.clients[bot1].name, wins1);
        println!("{} won {} times.", self.clients[bot2].name, wins2);
        println!("There were {} ties.", ties);
        println!("CDF1: {:.8}", cdf1);
        println!("CDF2: {:.8}", cdf2);
        let duration = now.elapsed();
        let duration = duration.as_secs() as f64
            + duration.subsec_millis() as f64 / 1000f64;
        println!("Game finished in {:.2} s.", duration);
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
        if cmd == "exit" || cmd == "quit" {
            break;
        }
        if cmd == "ping" {
            state.print_messages();
            state.ping();
        }
        if cmd == "notimeout" {
            state.set_timeout(None);
            println!("Removing timeout.");
        }
        if cmd == "timeout" {
            let timeout: u64 = match chunks.next() {
                Some(line) => {
                    match line.parse() {
                        Ok(index) => index,
                        Err(_err) => {
                            println!("{} is not a number.", line);
                            continue;
                        }
                    }
                },
                None => {
                    println!("Timeout requires an argument.");
                    continue;
                }
            };
            state.set_timeout(Some(timeout));
            println!("Setting timeout to {} secs.", timeout);
        }
        if cmd == "battle" {
            let bot1: usize = match chunks.next() {
                Some(line) => {
                    match line.parse() {
                        Ok(index) => index,
                        Err(_err) => {
                            println!("{} is not a number.", line);
                            continue;
                        }
                    }
                },
                None => {
                    println!("Battle requires three arguments.");
                    continue;
                }
            };
            let bot2: usize = match chunks.next() {
                Some(line) => {
                    match line.parse() {
                        Ok(index) => index,
                        Err(_err) => {
                            println!("{} is not a number.", line);
                            continue;
                        }
                    }
                },
                None => {
                    println!("Battle requires three arguments.");
                    continue;
                }
            };
            let battles: usize = match chunks.next() {
                Some(line) => {
                    match line.parse() {
                        Ok(index) => index,
                        Err(_err) => {
                            println!("{} is not a number.", line);
                            continue;
                        }
                    }
                },
                None => {
                    println!("Battle requires three arguments.");
                    continue;
                }
            };
            state.long_battle(bot1, bot2, battles);
        }
    }
    state.shutdown();
    println!("goodbye");
}
