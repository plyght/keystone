use anyhow::Result;
use std::fs;
use std::process::{Command, Stdio};

pub async fn start(bind: &str) -> Result<()> {
    let pid_file = get_pid_file();

    if pid_file.exists() {
        let pid_str = fs::read_to_string(&pid_file)?;
        let pid: u32 = pid_str.trim().parse()?;

        if is_process_running(pid) {
            anyhow::bail!("Daemon already running with PID {}", pid);
        } else {
            println!("Removing stale PID file");
            fs::remove_file(&pid_file)?;
        }
    }

    println!("🚀 Starting Birch daemon on {}", bind);

    let exe = std::env::current_exe()?;
    let child = Command::new(exe)
        .arg("daemon-internal-run")
        .arg("--bind")
        .arg(bind)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    fs::write(&pid_file, child.id().to_string())?;
    println!("✅ Daemon started with PID {}", child.id());

    Ok(())
}

pub async fn stop() -> Result<()> {
    let pid_file = get_pid_file();

    if !pid_file.exists() {
        anyhow::bail!("Daemon is not running (no PID file found)");
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse()?;

    if !is_process_running(pid) {
        println!("Daemon not running (stale PID file)");
        fs::remove_file(&pid_file)?;
        return Ok(());
    }

    println!("🛑 Stopping daemon (PID {})", pid);

    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill").arg(pid.to_string()).status()?;
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()?;
    }

    fs::remove_file(&pid_file)?;
    println!("✅ Daemon stopped");

    Ok(())
}

pub async fn status() -> Result<()> {
    let pid_file = get_pid_file();

    if !pid_file.exists() {
        println!("❌ Daemon is not running");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse()?;

    if is_process_running(pid) {
        println!("✅ Daemon is running (PID {})", pid);
    } else {
        println!("❌ Daemon is not running (stale PID file)");
    }

    Ok(())
}

pub async fn run_daemon(bind: String) -> Result<()> {
    println!("Starting daemon on {}", bind);

    crate::signals::start_server(&bind).await?;

    Ok(())
}

pub fn get_pid_file() -> std::path::PathBuf {
    crate::config::Config::birch_dir().join("daemon.pid")
}

pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
}

pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub bind_address: String,
}

pub fn get_daemon_status() -> Result<DaemonStatus> {
    let pid_file = get_pid_file();
    let config = crate::config::Config::load()?;

    if !pid_file.exists() {
        return Ok(DaemonStatus {
            running: false,
            pid: None,
            bind_address: config.daemon_bind,
        });
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse()?;

    let running = is_process_running(pid);

    Ok(DaemonStatus {
        running,
        pid: if running { Some(pid) } else { None },
        bind_address: config.daemon_bind,
    })
}
