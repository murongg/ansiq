use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityTab {
    Cpu,
    Memory,
    Energy,
    Disk,
    Network,
}

impl ActivityTab {
    pub const ALL: [Self; 5] = [
        Self::Cpu,
        Self::Memory,
        Self::Energy,
        Self::Disk,
        Self::Network,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Energy => "Energy",
            Self::Disk => "Disk",
            Self::Network => "Network",
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(Self::Cpu)
    }

    pub fn index(self) -> usize {
        match self {
            Self::Cpu => 0,
            Self::Memory => 1,
            Self::Energy => 2,
            Self::Disk => 3,
            Self::Network => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessSample {
    pub name: String,
    pub pid: i32,
    pub user: String,
    pub kind: String,
    pub cpu_percent: f32,
    pub cpu_time_secs: u64,
    pub threads: u32,
    pub idle_wakeups: u64,
    pub gpu_percent: f32,
    pub gpu_time_secs: u64,
    pub memory_bytes: u64,
    pub compressed_bytes: u64,
    pub energy_impact: f32,
    pub energy_impact_avg: f32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResourceTotals {
    pub user_percent: f32,
    pub system_percent: f32,
    pub idle_percent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemorySummary {
    pub used_bytes: u64,
    pub app_bytes: u64,
    pub wired_bytes: u64,
    pub compressed_bytes: u64,
    pub cached_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActivitySummary {
    pub read_per_sec: u64,
    pub write_per_sec: u64,
    pub total_in: u64,
    pub total_out: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NetworkSummary {
    pub recv_per_sec: u64,
    pub send_per_sec: u64,
    pub total_recv: u64,
    pub total_send: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivitySnapshot {
    pub process_count: usize,
    pub thread_count: usize,
    pub cpu: ResourceTotals,
    pub memory: MemorySummary,
    pub disk: ActivitySummary,
    pub network: NetworkSummary,
    pub processes: Vec<ProcessSample>,
    pub warning: Option<String>,
}

impl ActivitySnapshot {
    pub fn placeholder() -> Self {
        Self {
            process_count: 0,
            thread_count: 0,
            cpu: ResourceTotals {
                user_percent: 0.0,
                system_percent: 0.0,
                idle_percent: 100.0,
            },
            memory: MemorySummary::default(),
            disk: ActivitySummary::default(),
            network: NetworkSummary::default(),
            processes: Vec::new(),
            warning: Some("Collecting activity samples…".to_string()),
        }
    }

    pub fn tab_titles() -> Vec<String> {
        ActivityTab::ALL
            .iter()
            .map(|tab| tab.title().to_string())
            .collect()
    }

    pub fn headers_for(&self, tab: ActivityTab) -> Vec<String> {
        match tab {
            ActivityTab::Cpu => vec![
                "Process Name".to_string(),
                "% CPU".to_string(),
                "CPU Time".to_string(),
                "Threads".to_string(),
                "Idle Wake Ups".to_string(),
                "Kind".to_string(),
                "PID".to_string(),
                "User".to_string(),
            ],
            ActivityTab::Memory => vec![
                "Process Name".to_string(),
                "Memory".to_string(),
                "Compressed".to_string(),
                "Threads".to_string(),
                "PID".to_string(),
                "User".to_string(),
            ],
            ActivityTab::Energy => vec![
                "Process Name".to_string(),
                "Energy Impact".to_string(),
                "12 hr Avg".to_string(),
                "Idle Wake Ups".to_string(),
                "PID".to_string(),
                "User".to_string(),
            ],
            ActivityTab::Disk => vec![
                "Process Name".to_string(),
                "Bytes Read".to_string(),
                "Bytes Written".to_string(),
                "Threads".to_string(),
                "PID".to_string(),
                "User".to_string(),
            ],
            ActivityTab::Network => vec![
                "Process Name".to_string(),
                "Bytes In".to_string(),
                "Bytes Out".to_string(),
                "PID".to_string(),
                "User".to_string(),
            ],
        }
    }

    pub fn processes_for(&self, tab: ActivityTab) -> Vec<ProcessSample> {
        let mut processes = self.processes.clone();
        processes.sort_by(|left, right| compare_for_tab(left, right, tab));
        processes
    }

    pub fn selected_process(&self, tab: ActivityTab, pid: Option<i32>) -> Option<ProcessSample> {
        let processes = self.processes_for(tab);
        pid.and_then(|selected| {
            processes
                .into_iter()
                .find(|process| process.pid == selected)
        })
        .or_else(|| self.processes_for(tab).into_iter().next())
    }

    pub fn rows_for(&self, tab: ActivityTab) -> Vec<Vec<String>> {
        self.processes_for(tab)
            .into_iter()
            .map(|process| match tab {
                ActivityTab::Cpu => vec![
                    process.name,
                    format!("{:.1}", process.cpu_percent),
                    format_duration(process.cpu_time_secs),
                    process.threads.to_string(),
                    process.idle_wakeups.to_string(),
                    process.kind,
                    process.pid.to_string(),
                    process.user,
                ],
                ActivityTab::Memory => vec![
                    process.name,
                    format_bytes(process.memory_bytes),
                    format_bytes(process.compressed_bytes),
                    process.threads.to_string(),
                    process.pid.to_string(),
                    process.user,
                ],
                ActivityTab::Energy => vec![
                    process.name,
                    format!("{:.1}", process.energy_impact),
                    format!("{:.1}", process.energy_impact_avg),
                    process.idle_wakeups.to_string(),
                    process.pid.to_string(),
                    process.user,
                ],
                ActivityTab::Disk => vec![
                    process.name,
                    format_bytes(process.disk_read_bytes),
                    format_bytes(process.disk_write_bytes),
                    process.threads.to_string(),
                    process.pid.to_string(),
                    process.user,
                ],
                ActivityTab::Network => vec![
                    process.name,
                    format_bytes(process.network_in_bytes),
                    format_bytes(process.network_out_bytes),
                    process.pid.to_string(),
                    process.user,
                ],
            })
            .collect()
    }
}

fn compare_for_tab(left: &ProcessSample, right: &ProcessSample, tab: ActivityTab) -> Ordering {
    let ordering = match tab {
        ActivityTab::Cpu => right.cpu_percent.total_cmp(&left.cpu_percent),
        ActivityTab::Memory => right.memory_bytes.cmp(&left.memory_bytes),
        ActivityTab::Energy => right.energy_impact.total_cmp(&left.energy_impact),
        ActivityTab::Disk => (right.disk_read_bytes + right.disk_write_bytes)
            .cmp(&(left.disk_read_bytes + left.disk_write_bytes)),
        ActivityTab::Network => (right.network_in_bytes + right.network_out_bytes)
            .cmp(&(left.network_in_bytes + left.network_out_bytes)),
    };

    ordering.then_with(|| left.name.cmp(&right.name))
}

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{hours}:{minutes:02}:{seconds:02}")
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}
