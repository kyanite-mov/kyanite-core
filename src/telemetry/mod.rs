// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

pub mod models;

// Conditionally alias the platform module as `sys`
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as sys;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as sys;

pub struct Telemetry {
    pub cpu: sys::cpu::CpuTel,
    pub drive: sys::drive::DriveTel,
    pub gpu: sys::gpu::GpuTel,
    pub memory: sys::memory::MemoryTel,
    pub network: sys::network::NetworkTel,
    pub process: sys::process::ProcessTel,
    pub system: sys::system::SystemTel,
}

impl Telemetry {
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

    /// Updates all telemetry collectors concurrently
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