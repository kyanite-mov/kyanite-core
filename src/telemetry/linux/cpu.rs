// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

use crate::telemetry::models::CpuTelSnapshot;

pub struct CpuTel {
    snapshot: CpuTelSnapshot,
    // current_total_ticks: Vec<i64>,
    // previous_total_ticks: Vec<i64>,
}

impl CpuTel {
    pub fn new() -> Self {
        Self {
            snapshot: CpuTelSnapshot::default(),
        }
    }

    /// Fetches new data and overwrites the cached snapshot
    pub fn update(&mut self) {
        self.snapshot = CpuTelSnapshot {
            ..Default::default()
        };
    }

    /// Returns a reference to the current cached snapshot
    pub fn snapshot(&self) -> &CpuTelSnapshot {
        &self.snapshot
    }
}
