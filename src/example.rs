use std::io::{self, BufRead, Write};
use std::process;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

const EMIT_INTERVAL: Duration = Duration::from_secs(5);
const INITIAL_MESSAGE: &str = "message";

enum Event {
    Tick,
    Received(String),
    InputClosed,
    InputError(io::Error),
}

fn run() -> io::Result<()> {
    let (tx, rx) = mpsc::channel::<Event>();

    start_ticker(tx.clone(), EMIT_INTERVAL);
    start_stdin_listener(tx);

    let mut current_message = String::from(INITIAL_MESSAGE);
    process_events(rx, &mut current_message)
}

fn start_ticker(tx: Sender<Event>, interval: Duration) {
    thread::spawn(move || loop {
        thread::sleep(interval);
        if tx.send(Event::Tick).is_err() {
            break;
        }
    });
}

fn start_stdin_listener(tx: Sender<Event>) {
    thread::spawn(move || {
        let stdin = io::stdin();
        for line_result in stdin.lock().lines() {
            match line_result {
                Ok(line) => {
                    if tx.send(Event::Received(line)).is_err() {
                        return;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Event::InputError(err));
                    return;
                }
            }
        }

        let _ = tx.send(Event::InputClosed);
    });
}

fn process_events(rx: Receiver<Event>, current_message: &mut String) -> io::Result<()> {
    let mut input_closed_logged = false;

    loop {
        match rx.recv() {
            Ok(Event::Tick) => emit_output(current_message)?,
            Ok(Event::Received(message)) => receive_input(message, current_message)?,
            Ok(Event::InputClosed) => {
                if !input_closed_logged {
                    log_stderr("Entrée standard fermée, émission du dernier message reçue.\n")?;
                    input_closed_logged = true;
                }
            }
            Ok(Event::InputError(err)) => return Err(err),
            Err(_) => return Ok(()),
        }
    }
}

fn receive_input(message: String, current_message: &mut String) -> io::Result<()> {
    *current_message = message;

    log_stderr(&format!("Réception de: {}\n", current_message))
}

fn log_stderr(message: &str) -> io::Result<()> {
    let mut err = io::stderr().lock();
    err.write_all(message.as_bytes())?;
    err.flush()?;
    Ok(())
}

fn emit_output(message: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(message.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Erreur: {err}");
        process::exit(1);
    }
}
