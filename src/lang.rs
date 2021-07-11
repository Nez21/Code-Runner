use super::TMPDIR;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use uuid::Uuid;

pub enum Lang {
    C,
    Cpp,
    Go,
    Rust,
    Python2,
    Python3,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    CompilerTimeError,
    RuntimeError,
    Ok,
}

impl Lang {
    fn get_extension(&self) -> &'static str {
        match self {
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Go => "go",
            Lang::Rust => "rs",
            Lang::Python2 | Lang::Python3 => "py",
        }
    }

    fn compile(&self, dir: PathBuf) -> (String, Vec<String>, PathBuf, Status, String) {
        let output = match self {
            Lang::C => Command::new("firejail")
                .arg("--quiet")
                .arg("--shell=none")
                .arg("--noroot")
                .arg("--net=none")
                .arg("--private")
                .arg("gcc")
                .arg("-o")
                .arg(format!("{}", dir.with_extension("").to_string_lossy()))
                .arg(format!("{}", dir.to_string_lossy()))
                .current_dir(format!("{}", TMPDIR.to_string_lossy()))
                .output()
                .expect("Error occured when compling C!"),
            Lang::Cpp => Command::new("firejail")
                .arg("--quiet")
                .arg("--shell=none")
                .arg("--noroot")
                .arg("--net=none")
                .arg("--private")
                .arg("g++")
                .arg("-o")
                .arg(format!("{}", dir.with_extension("").to_string_lossy()))
                .arg(format!("{}", dir.to_string_lossy()))
                .current_dir(format!("{}", TMPDIR.to_string_lossy()))
                .output()
                .expect("Error occured when compling C++!"),
            Lang::Go => Command::new("firejail")
                .arg("--quiet")
                .arg("--shell=none")
                .arg("--noroot")
                .arg("--net=none")
                .arg("--private")
                .arg("go")
                .arg("build")
                .arg(format!("{}", dir.to_string_lossy()))
                .current_dir(format!("{}", TMPDIR.to_string_lossy()))
                .output()
                .expect("Error occured when compling Golang!"),
            Lang::Rust => Command::new("firejail")
                .arg("--quiet")
                .arg("--shell=none")
                .arg("--noroot")
                .arg("--net=none")
                .arg("--private")
                .arg("rustc")
                .arg(format!("{}", dir.to_string_lossy()))
                .current_dir(format!("{}", TMPDIR.to_string_lossy()))
                .output()
                .expect("Error occured when compling Rust!"),
            Lang::Python2 => {
                return (
                    "python2".to_string(),
                    vec![format!("{}", dir.to_string_lossy())],
                    dir,
                    Status::Ok,
                    String::new(),
                )
            }
            Lang::Python3 => {
                return (
                    "python3".to_string(),
                    vec![format!("{}", dir.to_string_lossy())],
                    dir,
                    Status::Ok,
                    String::new(),
                )
            }
        };
        let compiler_err = String::from_utf8_lossy(&output.stderr);
        remove_file(dir.clone()).unwrap();
        if compiler_err.len() > 0 {
            return (
                String::new(),
                Vec::new(),
                dir,
                Status::CompilerTimeError,
                format!("{}", compiler_err),
            );
        }
        return (
            format!("{}", dir.with_extension("").to_string_lossy()),
            Vec::new(),
            dir.with_extension(""),
            Status::Ok,
            String::new(),
        );
    }

    fn write_to_tmp(&self, source_code: &str) -> PathBuf {
        let mut dir = TMPDIR.clone();
        dir.push(format!("{:?}.{}", Uuid::new_v4(), self.get_extension()));
        let mut tmp = File::create(dir.clone()).unwrap();
        write!(tmp, "{}", source_code).unwrap();
        drop(tmp);
        dir
    }

    pub fn execute_code(&self, source_code: &str, input: &str, time_limit: u8) -> (Status, String) {
        let dir = self.write_to_tmp(source_code);
        let (exe, args, file, status, message) = self.compile(dir);
        if status == Status::CompilerTimeError {
            return (status, message);
        }
        let (status, message) = run_in_sandbox(&exe, args, input, time_limit, i32::MAX);
        remove_file(file).unwrap();
        (status, message)
    }
}

fn run_in_sandbox(
    exe: &str,
    args: Vec<String>,
    input: &str,
    cpu_time_limit: u8,
    _memory_limit: i32,
) -> (Status, String) {
    let mut cmd = Command::new("firejail");
    cmd
        .arg("--quiet")
        .arg("--shell=none")
        .arg("--noroot")
        .arg("--net=none")
        .arg("--blacklist=/home/")
        .arg("--blacklist=/etc/")
        .arg("--blacklist=/boot/")
        .arg("--blacklist=/var/")
        .arg(format!("--rlimit-cpu={}", cpu_time_limit))
        // .arg(format!("--rlimit-as={}", memory_limit)) Currently not working with Golang
        .arg("--seccomp.keep=getcwd,getpid,rt_sigreturn,brk,close,sched_getaffinity,dup,mmap,getuid,rt_sigaction,set_robust_list,set_tid_address,rt_sigprocmask,pread64,sysinfo,gettid,getdents64,lseek,geteuid,sigaltstack,getrandom,clone,futex,arch_prctl,fcntl,poll,readlink,access,mprotect,munmap,write,prlimit64,newfstatat,getegid,exit_group,readlinkat,ioctl,openat,read,getgid")
        .arg(exe)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    if args.len() > 0 {
        cmd.args(args);
    }
    let mut process = cmd.spawn().unwrap();
    if input.len() > 0 {
        process
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    let output = process.wait_with_output().unwrap();
    let runtime_err = String::from_utf8_lossy(&output.stderr);
    if runtime_err.len() > 0 {
        return (Status::RuntimeError, format!("{}", runtime_err));
    }
    return (
        Status::Ok,
        format!("{}", String::from_utf8_lossy(&output.stdout)),
    );
}
