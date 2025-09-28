use std::env;
use crate::job::JobTable;


fn expand_cd_arg(s: &str) -> String {
    if s == "~" {
        return env::var("HOME").unwrap_or_else(|_| "/".to_string());
    }
    if let Some(rest) = s.strip_prefix("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        return format!("{}/{}", home, rest);
    }
    if let Some(var) = s.strip_prefix('$') {
        return env::var(var).unwrap_or_else(|_| s.to_string());
    }
    s.to_string()
}











pub enum BuiltinResult { Handled, NotHandled }

pub struct History {
    buf: Vec<String>, // we’ll keep up to 3 recent valid commands
}
impl History {
    pub fn new() -> Self { Self { buf: Vec::new() } }
    pub fn push_valid(&mut self, line: &str) {
        if line.trim().is_empty() { return; }
        self.buf.push(line.to_string());
        if self.buf.len() > 3 {
            let start = self.buf.len() - 3;
            self.buf = self.buf[start..].to_vec();
        }
    }
    pub fn last_three(&self) -> Vec<String> {
        self.buf.clone()
    }
}

pub fn try_builtin(_line: &str, tokens: &[String], jobs: &mut JobTable, hist: &mut History) -> BuiltinResult {
    if tokens.is_empty() { return BuiltinResult::NotHandled; }
    match tokens[0].as_str() {
        "exit" => { builtin_exit(jobs, hist); /* never returns */ }
        "cd"   => { builtin_cd(&tokens[1..]); return BuiltinResult::Handled; }
        "jobs" => { builtin_jobs(jobs); return BuiltinResult::Handled; }
        _ => BuiltinResult::NotHandled,
    }
}

fn builtin_exit(jobs: &mut JobTable, hist: &History) -> ! {
    // Wait for all running background jobs
    while jobs.has_active() {
        jobs.poll();
        // sleep a tiny amount to avoid busy loop; not

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    // Print last three valid commands (or fewer per spec)
    let last = hist.last_three();
    if last.is_empty() {
        println!("no valid commands");
    } else {
        if last.len() < 3 {
            // Spec: fewer than three -> print only the last valid command
            if let Some(cmd) = last.last() {
                println!("{}", cmd);
            }
        } else {
            // Exactly three kept: print all three
            for cmd in last {
                println!("{}", cmd);
            }
        }
    }
    std::process::exit(0);
}

fn builtin_cd(args: &[String]) {
    match args.len() {
        0 => {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
            if let Err(e) = env::set_current_dir(&home) {
                eprintln!("cd: {}", e);
            }
        }
        1 => {
            let target = expand_cd_arg(&args[0]);
            if let Err(e) = env::set_current_dir(&target) {
                eprintln!("cd: {}", e);
            }
        }
        _ => {
            eprintln!("cd: too many arguments");
        }
    }
}

fn builtin_jobs(jobs: &JobTable) {
    jobs.list_running();
}

















