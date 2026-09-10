// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

use crate::telemetry::models::NetworkTelSnapshot;

pub struct NetworkTel {
    snapshot: NetworkTelSnapshot,
}

impl NetworkTel {
    pub fn new() -> Self {
        Self {
            snapshot: NetworkTelSnapshot::default(),
        }
    }

    /// Fetches new data and overwrites the cached snapshot
    pub fn update(&mut self) {
        self.snapshot = NetworkTelSnapshot {
            ..Default::default()
        };
    }

    /// Returns a reference to the current cached snapshot
    pub fn snapshot(&self) -> &NetworkTelSnapshot {
        &self.snapshot
    }
}
