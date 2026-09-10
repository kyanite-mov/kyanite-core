// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

use crate::telemetry::models::MemoryTelSnapshot;

pub struct MemoryTel {
    snapshot: MemoryTelSnapshot,
}

impl MemoryTel {
    pub fn new() -> Self {
        Self {
            snapshot: MemoryTelSnapshot::default(),
        }
    }

    /// Fetches new data and overwrites the cached snapshot
    pub fn update(&mut self) {
        self.snapshot = MemoryTelSnapshot {
            ..Default::default()
        };
    }

    /// Returns a reference to the current cached snapshot
    pub fn snapshot(&self) -> &MemoryTelSnapshot {
        &self.snapshot
    }
}
