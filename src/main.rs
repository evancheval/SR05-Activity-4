use colored_text::Colorize;
use std::env;
use std::io::{self, stderr, BufRead, BufReader, Write};
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
        // Cette instruction attendra qu'il y ait quelque chose sur stdin avant de continuer
        if let Some(line_result) = BufReader::new(io::stdin().lock()).lines().next() {
        receive_input(args.program_number, line_result).unwrap_or_default();}
    });

    loop {
        emit_output(ORIGINAL_MESSAGE, args.program_number)?;
        thread::sleep(interval);
    }
}

fn receive_input(program_number: u64, line_result: io::Result<String>) -> io::Result<String> {
        // Pour simuler l'atomicité (empêcher l'émission de s'éxécuter en même temps)
        let _stdout = io::stdout().lock();

        let message = line_result?;
        write_to_stderr(&format!(
            "[{}] Réception du message: {}\n",
            program_number, message
        ))?;
        // check_atomicity_for("receive input", program_number)?;
        Ok(message.trim().to_string())
}

fn emit_output(message: &str, program_number: u64) -> io::Result<()> {
    // Pour simuler l'atomicité (empêcher la réception de s'éxécuter en même temps)
    // let _stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let message = format!("[{}] {}", program_number, message);
    write_to_stderr(&format!(
            "[{}] Emission du message: {}\n",
            program_number, message
        ))?;
    stdout.write_all(
        message
            .hex("00d5ff")
            .as_bytes(),
    )?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    // check_atomicity_for("emit output", program_number)?;
    Ok(())
}

fn check_atomicity_for(fun: &str, program_number: u64) -> io::Result<()> {
    write_to_stderr(format!("[{}] checking atomicity for {} ...\n", program_number, fun).green().as_str())?;
    thread::sleep(Duration::from_secs(5));
    write_to_stderr(format!("[{}] finished checking atomicity for {}.\n", program_number, fun).green().as_str())?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse()?;

    run(args)
}
