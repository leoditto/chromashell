use std::io::{self, Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};

pub struct Pty {
    master_fd: RawFd,
    child_pid: libc::pid_t,
    master_file: std::fs::File,
}

impl Pty {
    pub fn spawn_shell(cols: u16, rows: u16) -> io::Result<Self> {
        let mut master_fd: RawFd = 0;
        let mut slave_fd: RawFd = 0;

        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        ws.ws_col = cols;
        ws.ws_row = rows;

        let ret = unsafe { libc::openpty(&mut master_fd, &mut slave_fd, std::ptr::null_mut(), std::ptr::null_mut(), &mut ws) };
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }

        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }

        if pid == 0 {
            // Child process
            unsafe {
                libc::close(master_fd);
                libc::setsid();
                libc::ioctl(slave_fd, libc::TIOCSCTTY as libc::c_ulong, 0);
                libc::dup2(slave_fd, 0);
                libc::dup2(slave_fd, 1);
                libc::dup2(slave_fd, 2);
                if slave_fd > 2 {
                    libc::close(slave_fd);
                }

                // Get user's shell
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
                let shell_c = std::ffi::CString::new(shell.as_str()).unwrap();
                let arg0 = std::ffi::CString::new("-glowr").unwrap();
                libc::execl(shell_c.as_ptr(), arg0.as_ptr(), std::ptr::null::<libc::c_char>());
                libc::_exit(1);
            }
        }

        // Parent
        unsafe { libc::close(slave_fd) };

        // Set master to non-blocking
        unsafe {
            let flags = libc::fcntl(master_fd, libc::F_GETFL);
            libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        let master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };

        Ok(Self {
            master_fd,
            child_pid: pid,
            master_file,
        })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.master_file.read(buf)
    }

    pub fn write_input(&mut self, data: &[u8]) -> io::Result<()> {
        self.master_file.write_all(data)
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        let ws = libc::winsize {
            ws_col: cols,
            ws_row: rows,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &ws);
        }
    }

    pub fn is_alive(&self) -> bool {
        unsafe {
            let mut status: libc::c_int = 0;
            let ret = libc::waitpid(self.child_pid, &mut status, libc::WNOHANG);
            ret == 0
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.child_pid, libc::SIGTERM);
        }
    }
}
