// Copyright (c) Akeoot / Akeoott <contact@kyanite.mov>. Licensed under the GPL-3.0 Licence.
// See the LICENSE file in the repository root for full license text.

use crate::telemetry::models::{
    CpuCoreFrequency, CpuCoreTemperature, CpuCoreUsage, CpuTelSnapshot,
};
use log::{trace, warn};
use std::fs::{self, File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

const RAPL_DIR_PREFIXES: &[&str] = &["intel-rapl", "amd-rapl"];
const CPUINFO_PATH: &str = "/proc/cpuinfo";
const PROCSTAT_PATH: &str = "/proc/stat";
const HWMON_ROOT: &str = "/sys/class/hwmon";
const POWERCAP_ROOT: &str = "/sys/class/powercap";

pub struct CpuTel {
    snapshot: CpuTelSnapshot,
    prev_total_ticks: Vec<i64>,
    prev_core_ticks: Vec<Vec<i64>>,
    first_usage_read: bool,

    energy_path: Option<PathBuf>,
    prev_energy_uj: f64,
    prev_energy_time: Instant,
    rapl_discovered: bool,
}

impl Default for CpuTel {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuTel {
    pub fn new() -> Self {
        Self {
            snapshot: CpuTelSnapshot::default(),
            prev_total_ticks: Vec::new(),
            prev_core_ticks: Vec::new(),
            first_usage_read: true,

            energy_path: None,
            prev_energy_uj: 0.0,
            prev_energy_time: Instant::now(),
            rapl_discovered: false,
        }
    }

    pub fn update(&mut self) {
        trace!("Fetching all CPU info...");
        self.snapshot = self.fetch_cpu_info();
    }

    pub fn snapshot(&self) -> &CpuTelSnapshot {
        &self.snapshot
    }

    fn fetch_cpu_info(&mut self) -> CpuTelSnapshot {
        let (cpu_model, core_frequencies) = read_cpu_info().unwrap_or_else(|e| {
            warn!("Failed to read {CPUINFO_PATH}: {e}");
            ("Unknown CPU".to_owned(), Vec::new())
        });

        let (cpu_usage, core_usages) = self.read_cpu_usages();
        let (cpu_temperature, core_temperatures) = read_cpu_temps(core_usages.len());
        let power_draw = self.read_power_draw();

        let cpu_frequency = if core_frequencies.is_empty() {
            0.0
        } else {
            let sum: f64 = core_frequencies.iter().map(|c| c.frequency).sum();
            round_to(sum / core_frequencies.len() as f64, 3)
        };

        CpuTelSnapshot {
            cpu_model,
            cpu_frequency,
            cpu_usage,
            cpu_temperature,
            power_draw,
            core_frequencies,
            core_usages,
            core_temperatures,
        }
    }

    fn read_cpu_usages(&mut self) -> (i32, Vec<CpuCoreUsage>) {
        let (curr_total, curr_core) = match read_current_ticks() {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to read {PROCSTAT_PATH}: {e}");
                return (0, Vec::new());
            }
        };

        // First sample sets the deltas, report zero usage until we have a diff.
        if self.first_usage_read {
            self.first_usage_read = false;
            let usages = curr_core
                .iter()
                .enumerate()
                .map(|(i, _)| CpuCoreUsage {
                    core_index: i as i32,
                    usage: 0.0,
                })
                .collect();
            self.prev_total_ticks = curr_total;
            self.prev_core_ticks = curr_core;
            return (0, usages);
        }

        let total_usage = round_to(compute_usage(&curr_total, &self.prev_total_ticks), 0) as i32;

        let core_usages = curr_core
            .iter()
            .enumerate()
            .map(|(i, curr)| {
                let usage = match self.prev_core_ticks.get(i) {
                    Some(prev) => round_to(compute_usage(curr, prev), 0),
                    None => 0.0,
                };
                CpuCoreUsage {
                    core_index: i as i32,
                    usage,
                }
            })
            .collect();

        self.prev_total_ticks = curr_total;
        self.prev_core_ticks = curr_core;

        (total_usage, core_usages)
    }

    fn read_power_draw(&mut self) -> f64 {
        if self.energy_path.is_none() {
            if self.rapl_discovered {
                return 0.0;
            }
            self.energy_path = discover_rapl_path();
            self.rapl_discovered = self.energy_path.is_none();
            if self.energy_path.is_none() {
                return 0.0;
            }
        }

        let path = self
            .energy_path
            .as_ref()
            .expect("energy_path is Some at this point");

        let Ok(raw) = fs::read_to_string(path) else {
            return 0.0;
        };
        let Ok(energy_uj) = raw.trim().parse::<f64>() else {
            return 0.0;
        };

        let current_time = Instant::now();
        let mut power = 0.0;

        if self.prev_energy_uj > 0.0 {
            // The counter can wrap, clamp to zero to avoid a negative spike.
            let delta_uj = (energy_uj - self.prev_energy_uj).max(0.0);
            let delta_sec = current_time
                .duration_since(self.prev_energy_time)
                .as_secs_f64();
            if delta_sec > 0.0 {
                power = delta_uj / 1_000_000.0 / delta_sec;
            }
        }

        self.prev_energy_uj = energy_uj;
        self.prev_energy_time = current_time;

        round_to(power, 2)
    }
}

fn read_cpu_info() -> std::io::Result<(String, Vec<CpuCoreFrequency>)> {
    let reader = BufReader::new(File::open(CPUINFO_PATH)?);

    let mut cpu_model = String::from("Unknown CPU");
    let mut model_set = false;
    let mut frequencies = Vec::new();
    let mut core_index: i32 = 0;

    for line in reader.lines() {
        let line = line?;

        if !model_set && line.starts_with("model name") {
            if let Some(colon) = line.find(':') {
                cpu_model = line[colon + 1..].trim().to_owned();
                model_set = true;
            }
        } else if line.starts_with("cpu MHz")
            && let Some(colon) = line.find(':')
            && let Ok(frequency) = line[colon + 1..].trim().parse::<f64>()
        {
            frequencies.push(CpuCoreFrequency {
                core_index,
                frequency,
            });
            core_index += 1;
        }
    }

    Ok((cpu_model, frequencies))
}

fn read_current_ticks() -> std::io::Result<(Vec<i64>, Vec<Vec<i64>>)> {
    let reader = BufReader::new(File::open(PROCSTAT_PATH)?);

    let mut total = Vec::new();
    let mut cores = Vec::new();
    let mut first = true;

    for line in reader.lines() {
        let line = line?;
        if !line.starts_with("cpu") {
            continue;
        }

        let ticks: Vec<i64> = line
            .split_ascii_whitespace()
            .skip(1)
            .filter_map(|s| s.parse::<i64>().ok())
            .collect();

        if first {
            total = ticks;
            first = false;
        } else {
            cores.push(ticks);
        }
    }

    Ok((total, cores))
}

fn compute_usage(curr: &[i64], prev: &[i64]) -> f64 {
    let len = curr.len().min(prev.len());
    let curr = &curr[..len];
    let prev = &prev[..len];

    let total_curr: i64 = curr.iter().sum();
    let total_prev: i64 = prev.iter().sum();

    let diff_total = total_curr - total_prev;
    if diff_total <= 0 {
        return 0.0;
    }

    let diff_idle = idle_sum(curr) - idle_sum(prev);
    (diff_total - diff_idle) as f64 / diff_total as f64 * 100.0
}

#[inline]
fn idle_sum(ticks: &[i64]) -> i64 {
    match ticks.len() {
        5.. => ticks[3] + ticks[4],
        4 => ticks[3],
        _ => 0,
    }
}

fn read_cpu_temps(core_count: usize) -> (i32, Vec<CpuCoreTemperature>) {
    let mut overall = 0;
    let mut raw_temps: Vec<CpuCoreTemperature> = Vec::new();

    if let Ok(entries) = fs::read_dir(HWMON_ROOT) {
        for entry in entries.flatten() {
            let hwmon_dir = entry.path();
            let Ok(raw_name) = fs::read_to_string(hwmon_dir.join("name")) else {
                continue;
            };

            let (dev_overall, dev_temps) = match raw_name.trim() {
                "coretemp" => read_intel_temps(&hwmon_dir),
                "k10temp" => read_amd_temps(&hwmon_dir),
                _ => continue,
            };

            if dev_overall != 0 {
                overall = dev_overall;
            }
            raw_temps.extend(dev_temps);
        }
    }

    (overall, uniform_temps(core_count, overall, raw_temps))
}

/// If the sensors line up with the core count, use them directly. Otherwise,
/// fill every core with the average temperature so callers see a stable array.
fn uniform_temps(
    core_count: usize,
    overall: i32,
    mut raw_temps: Vec<CpuCoreTemperature>,
) -> Vec<CpuCoreTemperature> {
    if !raw_temps.is_empty() && raw_temps.len() == core_count {
        raw_temps.sort_unstable_by_key(|t| t.core_index);
        return raw_temps;
    }

    let avg = if raw_temps.is_empty() {
        overall as f64
    } else {
        let sum: f64 = raw_temps.iter().map(|t| t.temperature as f64).sum();
        sum / raw_temps.len() as f64
    };

    let avg = avg.round() as i32;
    (0..core_count)
        .map(|i| CpuCoreTemperature {
            core_index: i as i32,
            temperature: avg,
        })
        .collect()
}

fn read_intel_temps(hwmon_dir: &Path) -> (i32, Vec<CpuCoreTemperature>) {
    let mut overall = 0;
    let mut temps = Vec::new();

    let Ok(entries) = fs::read_dir(hwmon_dir) else {
        return (overall, temps);
    };

    for (input_path, prefix) in sensor_inputs(entries) {
        let Some(temperature) = read_millideg(&input_path) else {
            continue;
        };
        let label = read_trimmed(hwmon_dir.join(format!("{prefix}_label")));

        match label.as_deref() {
            Some(l) if l.contains("Package") || l == "CPU" => overall = temperature,
            Some(l) if l.starts_with("Core ") => {
                let idx_str = l.rsplit(' ').next().unwrap_or("");
                if let Ok(idx) = idx_str.parse::<i32>() {
                    temps.push(CpuCoreTemperature {
                        core_index: idx,
                        temperature,
                    });
                }
            }
            _ => {}
        }
    }

    (overall, temps)
}

fn read_amd_temps(hwmon_dir: &Path) -> (i32, Vec<CpuCoreTemperature>) {
    let mut overall = 0;
    let mut has_tdie = false;
    let mut temps = Vec::new();

    let Ok(entries) = fs::read_dir(hwmon_dir) else {
        return (overall, temps);
    };

    for (input_path, prefix) in sensor_inputs(entries) {
        let Some(temperature) = read_millideg(&input_path) else {
            continue;
        };
        let label = read_trimmed(hwmon_dir.join(format!("{prefix}_label")));

        match label.as_deref() {
            Some(l) if l.contains("Tdie") => {
                overall = temperature;
                has_tdie = true;
            }
            Some(l) if l.contains("Tctl") && !has_tdie => overall = temperature,
            Some(l) if l.starts_with("Tccd") => {
                if let Ok(idx) = l[4..].parse::<i32>() {
                    temps.push(CpuCoreTemperature {
                        core_index: idx,
                        temperature,
                    });
                }
            }
            None if overall == 0 => overall = temperature,
            _ => {}
        }
    }

    (overall, temps)
}

/// Yields `(input_path, prefix)` for every `temp*_input` file in a hwmon dir.
fn sensor_inputs(entries: ReadDir) -> impl Iterator<Item = (PathBuf, String)> {
    entries.flatten().filter_map(|entry| {
        let name = entry.file_name();
        let name = name.to_str()?;
        let prefix = name.strip_suffix("_input")?;
        if !prefix.starts_with("temp") {
            return None;
        }
        Some((entry.path(), prefix.to_owned()))
    })
}

fn read_millideg(path: &Path) -> Option<i32> {
    let content = fs::read_to_string(path).ok()?;
    let millideg: i64 = content.trim().parse().ok()?;
    Some(millideg.div_euclid(1000) as i32)
}

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    Some(fs::read_to_string(path).ok()?.trim().to_owned())
}

fn discover_rapl_path() -> Option<PathBuf> {
    let entries = fs::read_dir(POWERCAP_ROOT).ok()?;

    let mut top_level: Option<PathBuf> = None;
    let mut sub_zone: Option<PathBuf> = None;

    for entry in entries.flatten() {
        let dir = entry.path();
        let Ok(dir_name) = entry.file_name().into_string() else {
            continue;
        };
        if !RAPL_DIR_PREFIXES.iter().any(|p| dir_name.starts_with(p)) {
            continue;
        }

        let energy_path = dir.join("energy_uj");
        if !energy_path.exists() {
            continue;
        }

        if dir_name.matches(':').count() > 1 {
            sub_zone.get_or_insert(energy_path);
            continue;
        }

        let name = read_trimmed(dir.join("name")).unwrap_or_default();
        if name.starts_with("package") {
            trace!("Discovered RAPL power domain: {}", energy_path.display());
            return Some(energy_path);
        }

        top_level.get_or_insert(energy_path);
    }

    let chosen = top_level.or(sub_zone);
    if let Some(ref p) = chosen {
        trace!("Using fallback RAPL domain: {}", p.display());
    }
    chosen
}

#[inline]
fn round_to(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}
