use std::{env, thread};
use std::fs::{File};
use std::io::{BufRead, BufReader, Read};
use std::os::fd::FromRawFd;
use std::process::{Command, exit};
use std::time::Duration;

use daemonize::{Daemonize, Outcome};
use nix::unistd::{close, dup2};

use exception::{GlobalError, GlobalResult};
use exception::anyhow::anyhow;

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
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to send TERM signal\nThe service may be down."))
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
    // 获取当前用户和组的 ID
    let uid = users::get_current_uid();
    let gid = users::get_current_gid();

    // 创建管道
    let mut pipe_fds = [0; 2];
    unsafe {
        if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
            eprintln!("Failed to create pipe");
            exit(1);
        }
    }

    let reader_fd = pipe_fds[0];
    let writer_fd = pipe_fds[1];

    //在 Unix 系统中，fork() 调用会复制当前进程的资源，包括代码、内存、文件描述符等。父子进程在 fork() 后共享代码，但会根据 fork() 的返回值进入不同的逻辑分支：
    let daemonize = Daemonize::new()
        .pid_file(exe_path.with_extension("pid"))
        .chown_pid_file(true)
        .working_directory(wd)
        .user(uid) // 设置用户权限
        .group(gid)
        .privileged_action(move || {
            // 将管道写端重定向到 stdout 和 stderr
            dup2(writer_fd, libc::STDOUT_FILENO).expect("Failed to redirect stdout");
            dup2(writer_fd, libc::STDERR_FILENO).expect("Failed to redirect stderr");
            close(writer_fd).expect("Failed to close writer_fd");
            D::init_privilege()
        });

    let mut first_child_exit_code = 0;
    let res = match daemonize.execute() {
        Outcome::Child(Ok(child)) => {
            Some(child.privileged_action_result)
        }
        Outcome::Child(Err(err)) => {
            eprintln!("Start middle process error: {}", err);
            Some(Err(GlobalError::SysErr(anyhow!(err))))
        }
        Outcome::Parent(Err(err)) => {
            eprintln!("Start main process error: {}", err);
            Some(Err(GlobalError::SysErr(anyhow!(err))))
        }
        Outcome::Parent(Ok(parent)) => {
            first_child_exit_code = parent.first_child_exit_code;
            println!("...");
            None
        }
    };
    match res {
        None => {
            // 关闭管道的写端
            close(writer_fd).expect("Failed to close writer_fd in parent");
            // 读取守护进程日志
            read_pipe_logs(reader_fd);
            exit(first_child_exit_code);
        }
        Some(Ok((d, t))) => {
            match d.run_app(t) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("App start run error: {}", e);
                    exit(first_child_exit_code);
                }
            }
        }
        Some(Err(e)) => {
            eprintln!("App init error: {}", e);
            exit(first_child_exit_code);
        }
    }
}

// 管道日志读取函数
fn read_pipe_logs(reader_fd: i32) {
    let reader = unsafe { File::from_raw_fd(reader_fd) };
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        match line {
            Ok(content) => println!("Daemon log: {}", content),
            Err(e) => eprintln!("Error reading pipe: {}", e),
        }
    }
}

fn stop_service() -> bool {
    let mut b = false;
    match read_pid() {
        None => {
            eprintln!("Service is not running\n   ...failed");
        }
        Some(pid) => {
            if let Err(e) = send_terminate_signal(pid) {
                eprintln!("Failed to stop the service: {}", e);
            } else {
                eprintln!("stop...\n   ...success");
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
    println!("restart ...");
    if stop_service() {
        thread::sleep(Duration::from_secs(1));
        start_service::<D, T>();
    }
}

pub fn run<D, T>()
where
    D: Daemon<T>,
{
    let arg_matches = cfg_lib::conf::get_arg_match();
    match arg_matches.subcommand() {
        Some(("start", args)) => {
            let config_path = args.try_get_one::<String>("config").expect("get config failed").expect("not found config").to_string();
            cfg_lib::conf::init_cfg(config_path);
            start_service::<D, T>();
        }
        Some(("stop", _)) => {
            stop_service();
        }
        Some(("restart", args)) => {
            let config_path = args.try_get_one::<String>("config").expect("get config failed").expect("not found config").to_string();
            cfg_lib::conf::init_cfg(config_path);
            restart_service::<D, T>();
        }
        _other => {
            eprintln!("Please add subcommands to operate: [start|stop|restart]")
        }
    }
}