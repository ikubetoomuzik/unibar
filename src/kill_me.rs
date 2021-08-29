//! Module to kill a process with a list of ids
//!
//!

use std::process::Command;

pub struct KillMeModule {
    command: String,
    args: Vec<String>,
    list: Vec<u32>,
}

impl KillMeModule {
    pub fn new(cmd: String) -> Self {
        let mut command = String::new();
        let mut args = Vec::new();
        for (i, s) in cmd
            .split_whitespace()
            .enumerate()
            .map(|(i, s)| (i, s.to_owned()))
        {
            if i == 0 {
                command = s;
            } else {
                args.push(s);
            }
        }
        Self {
            command,
            args,
            list: Vec::new(),
        }
    }

    pub fn kill_all(&mut self) {
        for id in self.list.drain(..) {
            match Command::new(&self.command)
                .args(&self.args)
                .arg(id.to_string())
                .output()
            {
                Ok(out) if out.status.success() => eprintln!("Killed process: {}", id),
                Ok(_) => eprintln!("Failed to kill process: {}", id),
                Err(e) => eprintln!("Error trying to kill process (id: {}): {}", id, e),
            }
        }
    }

    pub fn push(&mut self, id: u32) {
        if !self.list.contains(&id) {
            self.list.push(id);
        }
    }
}
