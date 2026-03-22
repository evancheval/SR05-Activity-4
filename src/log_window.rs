use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::sync::{Mutex, OnceLock};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

static LOG_FILE: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();
static LOG_VIEWER_CHILD: OnceLock<Mutex<Option<std::process::Child>>> = OnceLock::new();

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

// Initialise le fichier de logs et ouvre une fenêtre séparée pour visualiser les logs en temps réel.
// - Sur Windows : petite fenêtre PowerShell (90x18) avec Get-Content -Wait
// - Sur Linux : xterm ou gnome-terminal (80x15) avec tail -f

pub fn append_log_message(message: &str) -> io::Result<()> {
    if let Some(log_file) = LOG_FILE.get() {
        if let Ok(mut log_file) = log_file.lock() {
            log_file.write_all(message.as_bytes())?;
            log_file.flush()?;
        }
    }
    Ok(())
}

pub fn init_external_log_window(program_number: u64) -> io::Result<()> {
    let log_path = std::env::temp_dir().join(format!("sr05_logs_{}.ansi.log", program_number));

    let log_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&log_path)?;

    let _ = LOG_FILE.set(Mutex::new(BufWriter::new(log_file)));

    #[cfg(windows)]
    spawn_windows_log_viewer(&log_path, program_number)?;

    #[cfg(target_os = "linux")]
    spawn_linux_log_viewer(&log_path, program_number)?;

    #[cfg(target_os = "macos")]
    spawn_macos_log_viewer(&log_path, program_number)?;

    Ok(())
}

#[cfg(windows)]
fn spawn_windows_log_viewer(log_path: &std::path::Path, program_number: u64) -> io::Result<()> {
    let script_path = std::env::temp_dir().join(format!("sr05_logs_{}_viewer.ps1", program_number));

    let escaped_log_path = log_path.display().to_string().replace('"', "`\"");
    let ps_script = format!(
        "$Host.UI.RawUI.WindowTitle = 'SR05 Logs [{n}]'\r\n\
         try {{ $Host.UI.RawUI.WindowSize = New-Object System.Management.Automation.Host.Size(90, 18) }} catch {{}}\r\n\
         try {{ $Host.UI.RawUI.BufferSize = New-Object System.Management.Automation.Host.Size(90, 5000) }} catch {{}}\r\n\
         Get-Content -Path \"{p}\" -Wait -Tail 120\r\n",
        n = program_number,
        p = escaped_log_path
    );

    fs::write(&script_path, ps_script)?;

    let child = std::process::Command::new("powershell")
        .args([
            "-NoExit",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_string_lossy().as_ref(),
        ])
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()?;

    let child_slot = LOG_VIEWER_CHILD.get_or_init(|| Mutex::new(None));
    if let Ok(mut child_slot) = child_slot.lock() {
        *child_slot = Some(child);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn spawn_linux_log_viewer(log_path: &std::path::Path, program_number: u64) -> io::Result<()> {
    let log_path_str = log_path.to_string_lossy().to_string();
    let title = format!("SR05 Logs [{}]", program_number);

    let child = std::process::Command::new("xterm")
        .args([
            "-title",
            &title,
            "-geometry",
            "80x15",
            "-e",
            &format!("tail -f '{}' ; read", log_path_str),
        ])
        .spawn()
        .or_else(|_| {
            std::process::Command::new("gnome-terminal")
                .args([
                    "--title",
                    &title,
                    "--",
                    "bash",
                    "-c",
                    &format!("tail -f '{}' ; read", log_path_str),
                ])
                .spawn()
        });

    if let Ok(child) = child {
        let child_slot = LOG_VIEWER_CHILD.get_or_init(|| Mutex::new(None));
        if let Ok(mut child_slot) = child_slot.lock() {
            *child_slot = Some(child);
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn spawn_macos_log_viewer(log_path: &std::path::Path, program_number: u64) -> io::Result<()> {
    let log_path_str = log_path.to_string_lossy().to_string();
    let applescript = format!(
        "tell app \"Terminal\"\n\
         activate\n\
         do script \"tail -f '{}'; echo Press Ctrl+C to quit\"\n\
         set title displays to \"SR05 Logs [{}]\"\n\
         end tell",
        log_path_str, program_number
    );

    let child = std::process::Command::new("osascript")
        .args(["-e", &applescript])
        .spawn();

    if let Ok(child) = child {
        let child_slot = LOG_VIEWER_CHILD.get_or_init(|| Mutex::new(None));
        if let Ok(mut child_slot) = child_slot.lock() {
            *child_slot = Some(child);
        }
    }

    Ok(())
}

fn shutdown_external_log_window() {
    if let Some(child_slot) = LOG_VIEWER_CHILD.get() {
        if let Ok(mut child_slot) = child_slot.lock() {
            if let Some(mut child) = child_slot.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

pub fn install_shutdown_handler() -> io::Result<()> {
    ctrlc::set_handler(|| {
        shutdown_external_log_window();
        std::process::exit(0);
    })
    .map_err(|e| io::Error::other(format!("Impossible d'installer le handler Ctrl+C: {e}")))
}
