use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::net::{TcpListener, ToSocketAddrs};
use std::io::Error as IoError;
use std::thread::{JoinHandle, spawn, yield_now};

use client::Client;

pub struct ListenMessage {
    pub desc: &'static str,
    pub err: Option<IoError>,
}
impl ListenMessage {
    pub fn new(desc: &'static str, err: IoError) -> ListenMessage {
        ListenMessage {
            desc,
            err: Some(err),
        }
    }
    pub fn new_str(desc: &'static str) -> ListenMessage {
        ListenMessage {
            desc,
            err: None,
        }
    }
}
pub struct ShutdownHandle {
    handle: JoinHandle<()>,
    send: Sender<()>,
}
impl ShutdownHandle {
    pub fn shutdown(self) {
        let _ = self.send.send(());
        let _ = self.handle.join();
    }
}

pub fn listen_thread<A: ToSocketAddrs + Send + 'static>(
    addr: A,
    new_clients: Sender<Client>,
    messages: Sender<ListenMessage>
) -> ShutdownHandle {
    let (shutdown_send, shutdown_recv) = channel();
    let handle = spawn(move || {
        let listen = match TcpListener::bind(addr) {
            Ok(listen) => listen,
            Err(err) => {
                messages.send(ListenMessage::new("Unable to start server", err)).unwrap();
                return;
            },
        };
        match listen.set_nonblocking(true) {
            Ok(()) => {},
            Err(err) => {
                messages.send(ListenMessage::new(
                        "Failed setting nonblocking", err)).unwrap();
                return;
            }
        }
        loop {
            match listen.accept() {
                Ok((stream, addr)) => {
                    match Client::new(addr, stream) {
                        Ok(client) => {
                            new_clients.send(client).unwrap();
                        },
                        Err(err) => {
                            messages.send(ListenMessage::new(
                                    "Handshake failed.", err)).unwrap();
                        },
                    };
                },
                Err(err) => {
                    if err.kind() == ::std::io::ErrorKind::WouldBlock {
                        yield_now();
                    } else {
                        messages.send(ListenMessage::new(
                                "Error while listening for new clients.", err)).unwrap();
                        return;
                    }
                },
            }
            match shutdown_recv.try_recv() {
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => {
                    messages.send(ListenMessage::new_str(
                            "Shutdown disconnected.")).unwrap();
                    return;
                },
                Ok(()) => {
                    return;
                },
            }
        }
    });
    ShutdownHandle {
        handle,
        send: shutdown_send,
    }
}

