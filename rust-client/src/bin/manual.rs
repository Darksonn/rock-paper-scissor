use std::io::{Error as IoError, ErrorKind, stdin};
use std::env::args;
use std::io::BufRead;

extern crate rust_client;

fn is_move(c: u8) -> bool {
    match c as char {
        'r' => true,
        'p' => true,
        's' => true,
        _ => false,
    }
}

fn main() -> Result<(), Box<::std::error::Error>> {
    let mut args = args();
    let _name = args.next().unwrap();
    let ip = match args.next() {
        Some(ip) => ip,
        None => {
            println!("Ip required.");
            return Ok(());
        }
    };
    let stdin = stdin();
    let mut stdin = stdin.lock().lines();
    println!("Please choose a name:");
    let name = stdin.next().unwrap()?;
    let mut connection = rust_client::Connection::connect(ip, &name)?;
    println!("Connected!");
    loop {
        println!("Awaiting start of battle.");
        let start = connection.next_char()?;
        if start == 'x' as u8 {
            break;
        }
        if start != 'n' as u8 {
            return Err(IoError::new(ErrorKind::InvalidData,
                             "Server didn't start match").into());
        }
        println!("Game started, please select your move:");
        loop {
            let mut m = stdin.next().unwrap()?;
            while m.len() != 1 || !is_move(m.as_bytes()[0]) {
                m = stdin.next().unwrap()?;
            }
            connection.send_byte(m.as_bytes()[0])?;
            println!("Awaiting other player.");

            let response = connection.next_char()?;
            if 'A' as u8 <= response && 'Z' as u8 >= response {
                println!("Other player played {}.", (response ^ (' ' as u8)) as char);
                println!("End of game.");
                break;
            } else {
                println!("Other player played {}.", response as char);
            }
        }
    }
    println!("goodbye");
    Ok(())
}
