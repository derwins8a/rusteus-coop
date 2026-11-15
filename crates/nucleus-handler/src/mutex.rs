//! Mutex discovery and termination to allow concurrent game instances.

use nucleus_common::error::NucleusResult;
use nucleus_platform::MutexManager as PlatformMutexManager;

/// Manages system named mutexes, semaphores, and events to permit multi-instance execution.
pub struct MutexManager;

impl MutexManager {
    /// Closes or terminates a named mutex held by a target process.
    pub fn kill_mutex(
        platform_mutex: &dyn PlatformMutexManager,
        pid: u32,
        mutex_name: &str,
        mutex_type: &str,
    ) -> NucleusResult<bool> {
        platform_mutex.kill_mutex(pid, mutex_name, mutex_type)
    }

    /// Searches and terminates any matching mutexes for the given process.
    pub fn kill_matching_mutexes(
        platform_mutex: &dyn PlatformMutexManager,
        pid: u32,
        mutex_patterns: &[String],
    ) -> NucleusResult<usize> {
        Self::kill_matching_mutexes_typed(platform_mutex, pid, mutex_patterns, "Mutant")
    }

    /// Searches and terminates any matching mutexes for the given process with a specific type.
    pub fn kill_matching_mutexes_typed(
        platform_mutex: &dyn PlatformMutexManager,
        pid: u32,
        mutex_patterns: &[String],
        mutex_type: &str,
    ) -> NucleusResult<usize> {
        let mut count = 0;
        for pattern in mutex_patterns {
            if platform_mutex.kill_mutex(pid, pattern, mutex_type)? {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Polls a process for matching mutexes and terminates them as soon as they appear,
    /// retrying up to `max_wait` duration (at `poll_interval`).
    pub fn wait_and_kill_matching_mutexes(
        platform_mutex: &dyn PlatformMutexManager,
        pid: u32,
        mutex_patterns: &[String],
        max_wait: std::time::Duration,
        poll_interval: std::time::Duration,
    ) -> NucleusResult<usize> {
        Self::wait_and_kill_matching_mutexes_typed(
            platform_mutex,
            pid,
            mutex_patterns,
            "Mutant",
            max_wait,
            poll_interval,
        )
    }

    /// Polls a process for matching mutexes of a specific type and terminates them as soon as they appear,
    /// retrying up to `max_wait` duration (at `poll_interval`).
    pub fn wait_and_kill_matching_mutexes_typed(
        platform_mutex: &dyn PlatformMutexManager,
        pid: u32,
        mutex_patterns: &[String],
        mutex_type: &str,
        max_wait: std::time::Duration,
        poll_interval: std::time::Duration,
    ) -> NucleusResult<usize> {
        let start = std::time::Instant::now();
        let mut total_killed = 0;
        let mut remaining: Vec<String> = mutex_patterns.to_vec();

        while start.elapsed() < max_wait && !remaining.is_empty() {
            let mut newly_killed = Vec::new();
            for pattern in &remaining {
                match platform_mutex.kill_mutex(pid, pattern, mutex_type) {
                    Ok(true) => {
                        total_killed += 1;
                        newly_killed.push(pattern.clone());
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!("Error checking mutex '{}' for PID {}: {}", pattern, pid, e);
                    }
                }
            }

            remaining.retain(|p| !newly_killed.contains(p));
            if remaining.is_empty() {
                break;
            }

            std::thread::sleep(poll_interval);
        }

        Ok(total_killed)
    }
}
