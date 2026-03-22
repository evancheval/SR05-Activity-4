use colored_text::Colorize;
use std::env;
use std::io::{self, stderr, BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;

// Message fixé au départ
static ORIGINAL_MESSAGE: &str = "original message";
static LOG_COLOR: &str = "ff007f";
static STDOUT_OUTPUT: &str = "00d5ff";
static CHECKING_COLOR: &str = "08ff00";

struct Args {
    program_number: u64,
    test_atomicity: bool,
}

// Parse les arguments de la ligne de commande pour extraire le numéro du programme
// pour un affichage plus clair dans les logs. Ex: cargo run -- --program-number 1
impl Args {
    fn new() -> Self {
        Args {
            program_number: 0,
            test_atomicity: false,
        }
    }

    fn parse() -> io::Result<Self> {
        let args_iter: Vec<String> = env::args().collect();
        let mut args = Args::new();
        let mut i = 1;
        while i < args_iter.len() {
            match args_iter[i].as_str() {
                "--program-number" | "-p" => {
                    let val = args_iter.get(i + 1).ok_or_else(|| {
                        io::Error::other("--program-number/-p requiert une valeur entière.")
                    })?;
                    let n: u64 = val.parse().map_err(|_| {
                        io::Error::other(format!(
                        "Valeur invalide pour --program-number/-p: '{val}'. Entier positif attendu."
                    ))
                    })?;
                    args.program_number = n;
                    i += 1;
                }
                "--test-atomicity" => {
                    // Option de test pour simuler une vérification d'atomicité (ex: en vérifiant que les logs d'émission et de réception ne se mélangent pas)
                    args.test_atomicity = true;
                }
                _ => {
                    return Err(io::Error::other(format!(
                        "Argument inconnu: '{}'. Usage: cargo run -- --program-number/-p <entier>",
                        args_iter[i]
                    )))
                }
            }
            i += 1;
        }
        Ok(args)
    }
}

// Fonction utilitaire pour écrire dans stderr de manière plus concise
fn write_to_stderr(message: &str) -> io::Result<()> {
    let mut stderr = stderr();
    stderr.write_all(message.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

// Fonction principale qui gère l'émission périodique et la réception d'input
fn run(args: Args) -> io::Result<()> {
    let interval = Duration::from_secs(1);

    // Thread dédié à la réception d'input sur stdin
    thread::spawn(move || loop {
        // Cette instruction attendra qu'il y ait quelque chose sur stdin avant de continuer
        // (donc réception asynchrone car ne vérifie pas en permanence, mais attend passivement)
        if let Some(line_result) = BufReader::new(io::stdin().lock()).lines().next() {
            receive_input(line_result, args.program_number, args.test_atomicity)
                .unwrap_or_default();
        }
    });

    // Boucle principale d'émission périodique du message original
    loop {
        emit_output(ORIGINAL_MESSAGE, args.program_number, args.test_atomicity)?;
        thread::sleep(interval);
    }
}

// Fonction pour gérer la réception d'input, atomique
fn receive_input(
    line_result: io::Result<String>,
    program_number: u64,
    test_atomicity: bool,
) -> io::Result<String> {
    // Pour forcer l'atomicité (empêcher l'émission de s'éxécuter en même temps)
    let _stdout = io::stdout().lock();

    let message = line_result?;
    write_to_stderr(&format!("[{}] Réception du message: {}\n", program_number, message).hex(LOG_COLOR))?;
    if test_atomicity {
        check_atomicity_for("receive input", program_number)?;
    }
    Ok(message.trim().to_string())
}

fn emit_output(message: &str, program_number: u64, test_atomicity: bool) -> io::Result<()> {
    // Vérouiller au début du programme permet l'atomicté entre l'émission et la réception,
    // car la réception attendra que le verrou soit libéré avant de pouvoir s'exécuter, et vice versa.
    let mut stdout = io::stdout().lock();
    let message = format!("[{}] {}", program_number, message);
    write_to_stderr(&format!("[{}] Emission du message: {}\n", program_number, message).hex(LOG_COLOR))?;
    stdout.write_all(message.hex(STDOUT_OUTPUT).as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    if test_atomicity {
        check_atomicity_for("emit output", program_number)?;
    }
    Ok(())
}

// Fonction pour simuler une vérification d'atomicité (ex: en vérifiant que les logs d'émission et de réception ne se mélangent pas)
fn check_atomicity_for(fun: &str, program_number: u64) -> io::Result<()> {
    write_to_stderr(
        format!("[{}] checking atomicity for {} ...\n", program_number, fun)
            .hex(CHECKING_COLOR)
            .as_str(),
    )?;
    thread::sleep(Duration::from_secs(5));
    write_to_stderr(
        format!(
            "[{}] finished checking atomicity for {}.\n",
            program_number, fun
        )
        .hex(CHECKING_COLOR)
        .as_str(),
    )?;
    Ok(())
}

// Fonction pour afficher une légende des couleurs utilisées dans les logs
fn write_legend(program_number: u64) -> io::Result<()> {
    write_to_stderr("\n-------------------\n")?;
    write_to_stderr(format!("[{}] Color legend : \n", program_number).as_str())?;
    write_to_stderr("Checking\n".hex(CHECKING_COLOR).as_str())?;
    write_to_stderr("Log d'émission/de réception\n".hex(LOG_COLOR).as_str())?;
    write_to_stderr(
        "Message sur la sortie standard (sdout)\n"
            .hex(STDOUT_OUTPUT)
            .as_str(),
    )?;
    write_to_stderr("-------------------\n\n")?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse()?;
    write_legend(args.program_number)?;

    run(args)
}
