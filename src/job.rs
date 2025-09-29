use libc::{waitpid, WNOHANG};
#[derive(Clone)]
pub struct Job {
    pub id: usize,
    pub pid: i32,
    pub cmdline: String,
    pub running: bool,
}

pub struct JobTable {
    next_id: usize,      // starts at 1; monotonically increases; never reused
    jobs: Vec<Job>,      // we keep completed too, but mark running=false
}

impl JobTable {
    pub fn new() -> Self {
        Self { next_id: 1, jobs: Vec::new() }
    }

    pub fn has_active(&self) -> bool {
        self.jobs.iter().any(|j| j.running)
    }

    // pub fn running_pids(&self) -> Vec<i32> {
    //     self.jobs.iter().filter(|j| j.running).map(|j| j.pid).collect()
    // }

    /// Add a background job if under the 10-job limit.
    /// Prints: `[job_id] PID`
    pub fn add_job(&mut self, pid: i32, cmdline: String) -> Option<usize> {
        let active = self.jobs.iter().filter(|j| j.running).count();
        if active >= 10 {
            eprintln!("too many background processes (max 10)");
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.push(Job { id, pid, cmdline, running: true });
        println!("[{}] {}", id, pid);
        Some(id)
    }

    /// Print list of active background processes or “no active …”
    /// Format: `[Job number]+ [PID] [CMDLINE]`
    pub fn list_running(&self) {
        let running: Vec<&Job> = self.jobs.iter().filter(|j| j.running).collect();
        if running.is_empty() {
            println!("no active background processes");
        } else {
            for j in running {
                println!("[{}]+ {} {}", j.id, j.pid, j.cmdline);
            }
        }
    }

    /// Poll all children non-blockingly. When a child exits, mark done and print:
    /// `[job_id] + done CMDLINE`
    pub fn poll(&mut self) {
        // Loop until no more children report a change.
        loop {
            let mut status: i32 = 0;
            let ret = unsafe { waitpid(-1, &mut status as *mut i32, WNOHANG) };
            if ret <= 0 {
                break; // 0 = nothing changed; -1 = no children
            }
            let pid = ret;
            // Find job and mark as done, print completion
            if let Some(job) = self.jobs.iter_mut().find(|j| j.pid == pid && j.running) {
                job.running = false;
                println!("[{}] + done {}", job.id, job.cmdline);
            }
        }
    }
}




































































