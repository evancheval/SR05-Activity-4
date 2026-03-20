use colored_text::Colorize;
use std::env;
use std::io::{self, stderr, Read, Write};
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

    thread::spawn(move || loop {
        receive_input(args.program_number).unwrap_or_default();
    });

    loop {
        emit_output(ORIGINAL_MESSAGE, args.program_number)?;
        thread::sleep(interval);
    }
}

fn receive_input(program_number: u64) -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "[{}] Aucune donnée reçue sur l'entrée standard (stdin).",
                program_number
            ),
        ));
    }

    write_to_stderr(format!("[{}] Réception du message: {}\n", program_number, input).as_str())?;

    Ok(input.trim().to_string())
}

fn emit_output(message: &str, program_number: u64) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(
        format!("[{}] {}", program_number, message)
            .hex("00d5ff")
            .as_bytes(),
    )?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse()?;

    run(args)
}
