use std::{collections::HashMap, process::Command};

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

use crate::activity_monitor::{
    ActivitySnapshot, ActivitySummary, MemorySummary, NetworkSummary, ProcessSample, ResourceTotals,
};

#[derive(Clone, Debug, Default)]
struct PsSample {
    cpu_percent: f32,
    cpu_time_secs: u64,
    threads: u32,
    user: String,
}

#[derive(Clone, Debug, Default)]
struct NetworkSample {
    bytes_in: u64,
    bytes_out: u64,
}

pub struct ActivitySampler {
    system: System,
    energy_history: HashMap<i32, f32>,
}

impl ActivitySampler {
    pub fn new() -> Self {
        let refresh = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::everything());
        let mut system = System::new_with_specifics(refresh);
        system.refresh_all();

        Self {
            system,
            energy_history: HashMap::new(),
        }
    }

    pub fn collect(&mut self) -> Result<ActivitySnapshot, String> {
        self.system.refresh_all();

        let ps_samples = collect_ps_samples().unwrap_or_default();
        let network_samples = collect_network_samples().unwrap_or_default();
        let cpu_totals = collect_cpu_totals().unwrap_or_else(|_| ResourceTotals {
            user_percent: self.system.global_cpu_usage(),
            system_percent: 0.0,
            idle_percent: (100.0 - self.system.global_cpu_usage()).max(0.0),
        });
        let memory_summary = collect_memory_summary(&self.system)
            .unwrap_or_else(|_| fallback_memory_summary(&self.system));

        let mut processes = Vec::new();
        for process in self.system.processes().values() {
            let pid = process.pid().as_u32() as i32;
            let ps = ps_samples.get(&pid).cloned().unwrap_or_default();
            let net = network_samples.get(&pid).cloned().unwrap_or_default();
            let disk_usage = process.disk_usage();
            let memory_bytes = process.memory().saturating_mul(1024);

            let energy_now = ps.cpu_percent * 0.6
                + (ps.threads as f32 * 0.15)
                + bytes_to_mb(net.bytes_out + net.bytes_in) * 0.05;
            let energy_avg = match self.energy_history.get(&pid).copied() {
                Some(previous) => (previous * 0.8) + (energy_now * 0.2),
                None => energy_now,
            };
            self.energy_history.insert(pid, energy_avg);

            processes.push(ProcessSample {
                name: process.name().to_string_lossy().into_owned(),
                pid,
                user: ps.user,
                kind: "Apple".to_string(),
                cpu_percent: ps.cpu_percent.max(process.cpu_usage()),
                cpu_time_secs: ps.cpu_time_secs.max(process.run_time()),
                threads: ps.threads,
                idle_wakeups: 0,
                gpu_percent: 0.0,
                gpu_time_secs: 0,
                memory_bytes,
                compressed_bytes: 0,
                energy_impact: energy_now,
                energy_impact_avg: energy_avg,
                disk_read_bytes: disk_usage.total_read_bytes,
                disk_write_bytes: disk_usage.total_written_bytes,
                network_in_bytes: net.bytes_in,
                network_out_bytes: net.bytes_out,
            });
        }

        processes.retain(|process| !process.name.is_empty());

        let thread_count = processes
            .iter()
            .map(|process| process.threads as usize)
            .sum();
        let disk = ActivitySummary {
            read_per_sec: processes
                .iter()
                .map(|process| process.disk_read_bytes)
                .sum(),
            write_per_sec: processes
                .iter()
                .map(|process| process.disk_write_bytes)
                .sum(),
            total_in: 0,
            total_out: 0,
        };
        let network = NetworkSummary {
            recv_per_sec: processes
                .iter()
                .map(|process| process.network_in_bytes)
                .sum(),
            send_per_sec: processes
                .iter()
                .map(|process| process.network_out_bytes)
                .sum(),
            total_recv: processes
                .iter()
                .map(|process| process.network_in_bytes)
                .sum(),
            total_send: processes
                .iter()
                .map(|process| process.network_out_bytes)
                .sum(),
        };

        Ok(ActivitySnapshot {
            process_count: processes.len(),
            thread_count,
            cpu: cpu_totals,
            memory: memory_summary,
            disk,
            network,
            processes,
            warning: None,
        })
    }
}

fn collect_ps_samples() -> Result<HashMap<i32, PsSample>, String> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,pcpu=,time=,thcount=,user=,comm="])
        .output()
        .map_err(|error| format!("ps failed: {error}"))?;
    if !output.status.success() {
        return Err("ps returned a non-zero status".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut samples = HashMap::new();
    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let Some(pid) = parts.next().and_then(|value| value.parse::<i32>().ok()) else {
            continue;
        };
        let cpu_percent = parts
            .next()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.0);
        let cpu_time_secs = parts.next().map(parse_ps_duration).unwrap_or(0);
        let threads = parts
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        let user = parts.next().unwrap_or_default().to_string();

        samples.insert(
            pid,
            PsSample {
                cpu_percent,
                cpu_time_secs,
                threads,
                user,
            },
        );
    }

    Ok(samples)
}

fn collect_network_samples() -> Result<HashMap<i32, NetworkSample>, String> {
    let output = Command::new("nettop")
        .args(["-L", "1", "-P", "-x", "-J", "bytes_in,bytes_out"])
        .output()
        .map_err(|error| format!("nettop failed: {error}"))?;
    if !output.status.success() {
        return Err("nettop returned a non-zero status".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut samples = HashMap::new();
    for line in stdout.lines().skip(1) {
        let columns: Vec<_> = line.split(',').collect();
        if columns.len() < 4 {
            continue;
        }

        let Some((_, pid)) = columns[0].rsplit_once('.') else {
            continue;
        };
        let Some(pid) = pid.parse::<i32>().ok() else {
            continue;
        };
        let bytes_in = columns[2].parse::<u64>().unwrap_or(0);
        let bytes_out = columns[3].parse::<u64>().unwrap_or(0);
        samples.insert(
            pid,
            NetworkSample {
                bytes_in,
                bytes_out,
            },
        );
    }

    Ok(samples)
}

fn collect_cpu_totals() -> Result<ResourceTotals, String> {
    let output = Command::new("top")
        .args(["-l", "1", "-n", "0"])
        .output()
        .map_err(|error| format!("top failed: {error}"))?;
    if !output.status.success() {
        return Err("top returned a non-zero status".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cpu_line = stdout
        .lines()
        .find(|line| line.starts_with("CPU usage:"))
        .ok_or_else(|| "CPU summary line missing".to_string())?;
    let cpu_line = cpu_line.trim_start_matches("CPU usage:").trim();
    let parts: Vec<_> = cpu_line.split(',').collect();
    if parts.len() < 3 {
        return Err("CPU summary format changed".to_string());
    }

    Ok(ResourceTotals {
        user_percent: parse_percent(parts[0]),
        system_percent: parse_percent(parts[1]),
        idle_percent: parse_percent(parts[2]),
    })
}

fn collect_memory_summary(system: &System) -> Result<MemorySummary, String> {
    let output = Command::new("vm_stat")
        .output()
        .map_err(|error| format!("vm_stat failed: {error}"))?;
    if !output.status.success() {
        return Err("vm_stat returned a non-zero status".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut page_size = 16384u64;
    let mut values = HashMap::new();

    for line in stdout.lines() {
        if let Some(size) = line
            .split("page size of ")
            .nth(1)
            .and_then(|value| value.split(' ').next())
            .and_then(|value| value.parse::<u64>().ok())
        {
            page_size = size;
            continue;
        }

        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let cleaned = value.trim().trim_end_matches('.').replace('.', "");
        let Some(count) = cleaned.parse::<u64>().ok() else {
            continue;
        };
        values.insert(name.trim().to_string(), count);
    }

    Ok(MemorySummary {
        used_bytes: system.used_memory().saturating_mul(1024),
        app_bytes: values
            .get("Anonymous pages")
            .copied()
            .unwrap_or(0)
            .saturating_mul(page_size),
        wired_bytes: values
            .get("Pages wired down")
            .copied()
            .unwrap_or(0)
            .saturating_mul(page_size),
        compressed_bytes: values
            .get("Pages occupied by compressor")
            .copied()
            .unwrap_or(0)
            .saturating_mul(page_size),
        cached_bytes: values
            .get("File-backed pages")
            .copied()
            .unwrap_or(0)
            .saturating_mul(page_size),
        swap_used_bytes: system.used_swap().saturating_mul(1024),
    })
}

fn fallback_memory_summary(system: &System) -> MemorySummary {
    MemorySummary {
        used_bytes: system.used_memory().saturating_mul(1024),
        app_bytes: system.used_memory().saturating_mul(1024),
        wired_bytes: 0,
        compressed_bytes: 0,
        cached_bytes: system.available_memory().saturating_mul(1024),
        swap_used_bytes: system.used_swap().saturating_mul(1024),
    }
}

fn parse_percent(value: &str) -> f32 {
    value
        .split('%')
        .next()
        .unwrap_or_default()
        .trim()
        .parse::<f32>()
        .unwrap_or(0.0)
}

fn parse_ps_duration(value: &str) -> u64 {
    let pieces: Vec<_> = value.split(':').collect();
    match pieces.as_slice() {
        [hours, minutes, seconds] => {
            hours.parse::<u64>().unwrap_or(0) * 3600
                + minutes.parse::<u64>().unwrap_or(0) * 60
                + seconds.parse::<u64>().unwrap_or(0)
        }
        [minutes, seconds] => {
            minutes.parse::<u64>().unwrap_or(0) * 60 + seconds.parse::<u64>().unwrap_or(0)
        }
        _ => 0,
    }
}

fn bytes_to_mb(bytes: u64) -> f32 {
    bytes as f32 / (1024.0 * 1024.0)
}
