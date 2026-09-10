// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

//! Telemetry models for system monitoring.

macro_rules! model {
    ($($item:item)*) => {
        $(
            #[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
            $item
        )*
    }
}

// CPU Models
model! {
    /// A snapshot of all CPU metrics collected at a single point in time.
    pub struct CpuTelSnapshot {
        /// CPU model name string.
        pub cpu_model: String,
        /// Overall CPU frequency in MHz.
        pub cpu_frequency: f64,
        /// Overall CPU usage percentage.
        pub cpu_usage: i32,
        /// Overall CPU temperature in degrees Celsius.
        pub cpu_temperature: i32,
        /// Current CPU power draw in watts.
        pub power_draw: f64,
        /// Per-core frequency measurements.
        pub core_frequencies: Vec<CpuCoreFrequency>,
        /// Per-core usage percentages.
        pub core_usages: Vec<CpuCoreUsage>,
        /// Per-core or per-CCD temperature readings.
        pub core_temperatures: Vec<CpuCoreTemperature>,
    }

    /// Represents the measured clock speed of a single CPU core.
    pub struct CpuCoreFrequency {
        /// Zero-based core index.
        pub core_index: i32,
        /// Core frequency in MHz.
        pub frequency: f64
    }

    /// Represents the CPU usage percentage of a single core.
    pub struct CpuCoreUsage {
        /// Zero-based core index.
        pub core_index: i32,
        /// Core usage percentage (0-100).
        pub usage: f64
    }

    /// Represents the temperature reading of a single CPU core.
    pub struct CpuCoreTemperature {
        /// Zero-based core index (or CCD index on AMD).
        pub core_index: i32,
        /// Core temperature in degrees Celsius.
        pub temperature: i32
    }
}

// Drive Models
model! {
    /// A snapshot of all mounted filesystem metrics collected at a single point in time.
    pub struct DriveTelSnapshot {
        /// Array of mounted drive information entries.
        pub mounts: Vec<DriveMountInfo>,
    }

    /// Represents information about a single mounted filesystem.
    pub struct DriveMountInfo {
        /// Zero-based index in the mount array.
        pub mount_index: i32,
        /// Path to the mount point (e.g. "/").
        pub mount_point: String,
        /// Device name or path (e.g. "/dev/nvme0n1p2").
        pub device_name: String,
        /// Type of filesystem (e.g. "btrfs", "ext4", "ntfs").
        pub filesystem_type: String,
        /// Total capacity in bytes.
        pub total_bytes: i64,
        /// Available free space in bytes.
        pub available_bytes: i64,
        /// Used space in bytes.
        pub used_bytes: i64,
        /// Disk I/O usage percentage.
        pub io_usage: f64,
    }
}

// GPU Models
model! {
    /// A snapshot of all GPU metrics collected at a single point in time.
    pub struct GpuTelSnapshot {
        /// GPU model name.
        pub gpu_model: String,
        /// GPU core utilization percentage (0-100).
        pub gpu_usage: i32,
        /// GPU memory utilization percentage (0-100).
        pub memory_usage: i32,
        /// GPU memory used in megabytes.
        pub memory_used_mb: f64,
        /// Total GPU memory in megabytes.
        pub memory_total_mb: f64,
        /// GPU temperature in degrees Celsius.
        pub temperature: i32,
        /// GPU power state (e.g. P0, P2, P8).
        pub power_state: String,
        /// Current GPU power draw in watts.
        pub power_draw: f64,
    }

    /// Known GPU vendors for hardware-specific implementation selection.
    pub enum GpuVendor {
        /// GPU vendor could not be detected or is unsupported.
        #[default]
        Unknown,
        /// NVIDIA graphics hardware.
        Nvidia,
        /// AMD graphics hardware.
        Amd,
        /// Intel graphics hardware. Note: Intel GPU detection is not yet implemented. This member is reserved for future use.
        Intel,
    }
}

// Memory Models
model! {
    /// A snapshot of system memory metrics collected at a single point in time. All values are in gibibytes (GiB).
    pub struct MemoryTelSnapshot {
        /// Total physical memory in GiB.
        pub memory_total_gib: f64,
        /// Free (unused) physical memory in GiB.
        pub memory_free_gib: f64,
        /// Available memory including reclaimable cache in GiB.
        pub memory_available_gib: f64,
        /// Used physical memory (total - available) in GiB.
        pub memory_used_gib: f64,
        /// Cached memory in GiB.
        pub memory_cached_gib: f64,
        /// Total swap space in GiB.
        pub swap_total_gib: f64,
        /// Free swap space in GiB.
        pub swap_free_gib: f64,
    }
}

// Network Models
model! {
    /// Snapshot of all network interface metrics at a point in time.
    pub struct NetworkTelSnapshot {
        /// Aggregate download speed across all connected networks.
        pub download_bytes_per_sec: i64,
        /// Aggregate upload speed across all connected networks.
        pub upload_bytes_per_sec: i64,
        /// Per-interface network metrics.
        pub interfaces: Vec<ConnectedNetwork>,
    }

    /// Represents a connected network with its current transfer rates and cumulative byte counters along other details.
    pub struct ConnectedNetwork {
        /// The detected network name (e.g. eth0, wlan0).
        pub interface_name: String,
        /// Current download speed in bytes per second.
        pub download_bytes_per_sec: i64,
        /// Current upload speed in bytes per second.
        pub upload_bytes_per_sec: i64,
        /// Cumulative bytes received since boot.
        pub total_downloaded_bytes: i64,
        /// Cumulative bytes transmitted since boot.
        pub total_uploaded_bytes: i64,
        /// `true` if the detected network is administratively up.
        pub is_up: bool,
    }
}

// Process Models
model! {
    /// A snapshot of all processes and metadata obtained during a single monitoring tick.
    pub struct ProcessTelSnapshot {
        /// The total number of processes in the snapshot.
        pub total_processes: i32,
        /// The array of individual process details.
        pub processes: Vec<ProcessInfo>,
    }

    /// Provides a snapshot of a single process's identifier, metadata, and resource usage.
    pub struct ProcessInfo {
        /// The unique process identifier.
        pub pid: i32,
        /// The name of the executable or program.
        pub program: String,
        /// The full command line used to start the process.
        pub command: String,
        /// The name of the user who owns the process.
        pub user: String,
        /// The current execution state of the process.
        pub state: ProcessState,
        /// The scheduling priority of the process.
        pub priority: ProcessPriority,
        /// The number of threads in the process.
        pub thread_count: i32,
        /// The memory usage of the process in megabytes (MB).
        pub memory_usage_mb: i32,
        /// The current CPU usage percentage of the process.
        pub cpu_usage: f64,
    }

    /// Represents the current execution state of a process.
    #[repr(u8)]
    pub enum ProcessState {
        /// The process state could not be determined.
        #[default]
        Unknown,
        /// The process is currently running or ready to run.
        Running,
        /// The process is sleeping, idle, or waiting for resources (e.g., I/O).
        Sleeping,
        /// The process has terminated but its parent has not yet reaped it.
        Zombie,
        /// The process has been stopped or suspended.
        Stopped,
        /// The process is dead and should no longer appear in listings.
        Dead,
    }

    /// Defines the scheduling priority of a process, from lowest to highest.
    #[repr(u8)]
    pub enum ProcessPriority {
        /// Unknown priority state.
        #[default]
        Unknown,
        /// Lowest priority, runs only when the system is idle.
        Idle,
        /// Priority below normal (e.g., positive nice value on Unix systems).
        BelowNormal,
        /// Default or normal scheduling priority.
        Normal,
        /// Priority above normal (e.g., negative nice value on Unix systems).
        AboveNormal,
        /// High priority, reserved for time-critical tasks.
        High,
        /// Highest priority, real-time scheduling. May require root privileges.
        RealTime,
    }
}

// System Models
model! {
    /// A snapshot of system-level information collected at a single point in time.
    pub struct SystemTelSnapshot {
        /// Operating system kernel version string.
        pub kernel_version: String,
        /// System hostname.
        pub hostname: String,
        /// System uptime in seconds.
        pub uptime_seconds: f64,
        /// Number of currently running tasks/processes.
        pub running_task_count: i32,
        /// Total number of tasks/processes on the system.
        pub total_task_count: i32,
    }
}
