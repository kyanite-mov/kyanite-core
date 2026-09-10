// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

use std::thread;
use std::time::Duration;
use kyanite_core::Telemetry;

fn main() {
    // 1. Initialize the aggregate (everything is zeroed/empty)
    let mut telemetry = Telemetry::new();

    // 2. Refresh all hardware data twice
    println!("Updating all");
    telemetry.update_all();
    println!("Done, waiting one second...");
    thread::sleep(Duration::from_secs(1));
    println!("Updating again\n");
    telemetry.update_all();

    // 3. Read and print snapshots
    println!("{:?}", telemetry.cpu.snapshot());
    println!("{:?}", telemetry.drive.snapshot());
    println!("{:?}", telemetry.gpu.snapshot());
    println!("{:?}", telemetry.memory.snapshot());
    println!("{:?}", telemetry.network.snapshot());
    println!("{:?}", telemetry.process.snapshot());
    println!("{:?}", telemetry.system.snapshot());
}
