use colored_text::Colorize;
use std::env;
use std::io::{self, stderr, BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

static ORIGINAL_MESSAGE: &str = "original message";

struct Args {
    program_number: u64,
}

impl Args {
    fn parse() -> io::Result<Self> {
        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            if args[i] == "--program-number" || args[i] == "-p" {
                let val = args.get(i + 1).ok_or_else(|| {
                    io::Error::other("--program-number/-p requiert une valeur entière.")
                })?;
                let n: u64 = val.parse().map_err(|_| {
                    io::Error::other(format!(
                        "Valeur invalide pour --program-number/-p: '{val}'. Entier positif attendu."
                    ))
                })?;
                return Ok(Self { program_number: n });
            }
            i += 1;
        }
        Err(io::Error::other(
            "Argument manquant. Usage: cargo run -- --program-number/-p <entier>",
        ))
    }
}

fn write_to_stderr(message: &str) -> io::Result<()> {
    let mut stderr = stderr();
    stderr.write_all(message.red().as_bytes())?;
    stderr.flush()?;
    Ok(())
}

fn run(args: Args) -> io::Result<()> {
    let interval = Duration::from_secs(1);

    let (tx, rx) = mpsc::channel::<String>();

    let args_clone = Args {
        program_number: args.program_number,
    };
    thread::spawn(move || loop {
        if let Ok(msg) = receive_input(args_clone.program_number) {
            let _ = tx.send(msg);
        }
    });

    let mut current_message = ORIGINAL_MESSAGE.to_string();

    loop {
        // Vérifier s'il y a un nouveau message depuis stdin
        if let Ok(new_msg) = rx.try_recv() {
            current_message = new_msg;
            write_to_stderr(&format!(
                "[{}] Message mis à jour: {}\n",
                args.program_number, current_message
            ))?;
        }

        emit_output(&current_message, args.program_number)?;
        thread::sleep(interval);
    }
}

fn receive_input(program_number: u64) -> io::Result<String> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    write_to_stderr(&format!("[{}] Waiting for input\n", program_number))?;

    // Lire UNE SEULE ligne, pas boucler jusqu'à EOF
    let mut lines = reader.lines();
    if let Some(line_result) = lines.next() {
        let line = line_result?;
        write_to_stderr(&format!("[{}] Réception du message: {}\n", program_number, line))?;
        Ok(line.trim().to_string())
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "[{}] Aucune donnée reçue sur l'entrée standard (stdin).",
                program_number
            ),
        ))
    }
}

fn emit_output(message: &str, program_number: u64) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(
        format!("[{}] {}", program_number, message)
            .hex("00d5ff")
            .as_bytes(),
    )?;
    stdout.write_all(b"\r\n")?;
    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse()?;

    run(args)
}
