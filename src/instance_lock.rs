use anyhow::{anyhow, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

/// Single instance lock using flock(2)
pub struct InstanceLock {
    _file: File,
    lock_path: PathBuf,
}

impl InstanceLock {
    /// Try to acquire the instance lock.
    /// Returns Ok(lock) if successful, Err if another instance is running.
    pub fn acquire() -> Result<Self> {
        let lock_path = Self::lock_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open (or create) the lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;

        // Try to acquire exclusive lock (non-blocking)
        let fd = file.as_raw_fd();
        let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

        if result != 0 {
            return Err(anyhow!(
                "Another instance of MyBuds is already running.\n\
                 Only one instance is allowed at a time.\n\
                 Lock file: {}",
                lock_path.display()
            ));
        }

        // Write PID to lock file (for debugging)
        let mut file_clone = file.try_clone()?;
        let pid = std::process::id();
        writeln!(file_clone, "{}", pid)?;
        file_clone.flush()?;

        Ok(Self {
            _file: file,
            lock_path,
        })
    }

    fn lock_file_path() -> Result<PathBuf> {
        // Use XDG_RUNTIME_DIR if available (better for locks), fallback to /tmp
        let lock_dir = std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));

        Ok(lock_dir.join("mybuds.lock"))
    }
}

// Lock is automatically released when InstanceLock is dropped (file is closed)
impl Drop for InstanceLock {
    fn drop(&mut self) {
        // flock is automatically released when the file descriptor is closed
        // Delete the lock file to clean up
        let _ = std::fs::remove_file(&self.lock_path);
    }
}
