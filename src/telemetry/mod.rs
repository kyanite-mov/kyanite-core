// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

pub mod models;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as sys;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as sys;

/// Aggregates all telemetry collectors into a single entry point.
///
/// Each field is a platform-specific implementation of a telemetry
/// service. Call any or all update methods to populate their snapshots,
/// then read individual snapshots via each service's `snapshot()` method.
pub struct Telemetry {
    /// CPU telemetry collector.
    pub cpu: sys::cpu::CpuTel,
    /// Drive (storage) telemetry collector.
    pub drive: sys::drive::DriveTel,
    /// GPU telemetry collector.
    pub gpu: sys::gpu::GpuTel,
    /// Memory telemetry collector.
    pub memory: sys::memory::MemoryTel,
    /// Network telemetry collector.
    pub network: sys::network::NetworkTel,
    /// Process telemetry collector.
    pub process: sys::process::ProcessTel,
    /// System-level telemetry collector.
    pub system: sys::system::SystemTel,
}

impl Telemetry {
    /// Creates a new [`Telemetry`] instance with all snapshots initialized
    /// to their default (empty/zero) values.
    pub fn new() -> Self {
        Self {
            cpu: sys::cpu::CpuTel::new(),
            drive: sys::drive::DriveTel::new(),
            gpu: sys::gpu::GpuTel::new(),
            memory: sys::memory::MemoryTel::new(),
            network: sys::network::NetworkTel::new(),
            process: sys::process::ProcessTel::new(),
            system: sys::system::SystemTel::new(),
        }
    }

    /// Updates all telemetry collectors concurrently.
    ///
    /// Each collector runs on its own thread via `rayon::scope`, so a
    /// single call fetches all data in parallel.
    /// Blocks until all collectors have finished.
    pub fn update_all(&mut self) {
        rayon::scope(|s| {
            s.spawn(|_| self.cpu.update());
            s.spawn(|_| self.drive.update());
            s.spawn(|_| self.gpu.update());
            s.spawn(|_| self.memory.update());
            s.spawn(|_| self.network.update());
            s.spawn(|_| self.process.update());
            self.system.update();
        });
    }
}
