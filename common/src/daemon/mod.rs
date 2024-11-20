use std::{env, thread};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::process::{Command};
use std::time::Duration;

use daemonize::{Daemonize};

use exception::{GlobalResult};

pub trait Daemon<T> {
    fn init_privilege() -> GlobalResult<(Self, T)>
    where
        Self: Sized;
    fn run_app(self, t: T) -> GlobalResult<()>;
}

// 在32位系统中，32768是pid_max的最大值。64位系统，pid_max最大可达2^22。（PID_MAX_LIMIT，大小是4194304）
// cat /proc/sys/kernel/pid_max
fn read_pid() -> Option<i32> {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let pid_file_path = exe_path.with_extension("pid");
    if let Ok(mut file) = File::open(pid_file_path) {
        let mut pid_str = String::new();
        file.read_to_string(&mut pid_str).expect("读取pid信息失败");
        let pid = pid_str.trim().parse::<i32>().expect("invalid pid");
        return Some(pid);
    }
    None
}

fn send_terminate_signal(pid: i32) -> Result<(), std::io::Error> {
    Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()
        .map(|status| {
            if !status.success() {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to send TERM signal:Service maybe down"))
            } else {
                Ok(())
            }
        })?
}

fn start_service<D, T>()
where
    D: Daemon<T>,
{
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let wd = exe_path.parent().expect("invalid path");
    let out_path = exe_path.with_extension("err");
    let stderr = OpenOptions::new().create(true).write(true).append(true).open(out_path).expect("Failed to open run.out");
    // 获取当前用户和组的 ID
    let uid = users::get_current_uid();
    let gid = users::get_current_gid();

    let daemonize = Daemonize::new()
        .pid_file(exe_path.with_extension("pid"))
        .chown_pid_file(true)
        .working_directory(wd)
        .user(uid) // 设置用户权限
        .group(gid)
        .privileged_action(|| D::init_privilege())
        // .stdout(stdout.try_clone().expect("Failed to clone start log file handle"))  // 重定向 stdout 到文件
        .stderr(stderr); // 重定向 stdout 到文件
    match daemonize.start() {
        Ok(Ok((d, t))) => {
            eprintln!("Service started successfully...");
            d.run_app(t).unwrap()
        }
        Ok(Err(e)) => {
            eprintln!("Service started failed: {}.", e);
        }
        Err(e) => {
            eprintln!("Error starting the service:{}", e);
        }
    }
}

fn stop_service() -> bool {
    let mut b = false;
    match read_pid() {
        None => {
            eprintln!("Service is not running.");
        }
        Some(pid) => {
            if let Err(e) = send_terminate_signal(pid) {
                eprintln!("Failed to stop the service: {}", e);
            } else {
                eprintln!("Service stopped.");
                b = true;
            }
        }
    }
    return b;
}

fn restart_service<D, T>()
where
    D: Daemon<T>,
{
    if stop_service() {
        eprintln!("restart ...");
        thread::sleep(Duration::from_secs(1));
        start_service::<D, T>();
    }
}

pub fn run<D, T>()
where
    D: Daemon<T>,
{
    let arg_matches = cfg_lib::cache::get_arg_match();
    match arg_matches.subcommand() {
        Some(("start", args)) => {
            let config_path = args.try_get_one::<String>("config").expect("get config failed").expect("not found config").to_string();
            cfg_lib::cache::init_cfg(config_path);
            start_service::<D, T>();
        }
        Some(("stop", _)) => {
            stop_service();
        }
        Some(("restart", args)) => {
            let config_path = args.try_get_one::<String>("config").expect("get config failed").expect("not found config").to_string();
            cfg_lib::cache::init_cfg(config_path);
            restart_service::<D, T>();
        }
        _other => {
            eprintln!("Please add subcommands to operate: [start|stop|restart]")
        }
    }
}