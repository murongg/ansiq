use std::time::Duration;

use ansiq_core::{Constraint, Element, Layout, Length, TableAlignment, ViewCtx};
use ansiq_runtime::{App, RuntimeHandle};
use ansiq_surface::ViewportPolicy;
use ansiq_widgets::{Block, Box, Paragraph, Shell, StatusBar, Table, Tabs, Text};

use crate::activity_monitor::{
    ActivitySampler, ActivitySnapshot, ActivityTab, ProcessSample, format_bytes,
};

// Activity monitor is a fixed app shell, not a content-growing transcript.
// Keeping a stable viewport avoids footer border ghosts when the first real
// snapshot arrives and the process table becomes taller than the placeholder.
pub const VIEWPORT_POLICY: ViewportPolicy = ViewportPolicy::ReserveFitContent { min: 28, max: 28 };

#[derive(Clone, Debug)]
pub enum ActivityMonitorMessage {
    Snapshot(ActivitySnapshot),
    SelectTab(usize),
    SelectProcess(usize),
    SamplingFailed(String),
}

pub struct ActivityMonitorApp {
    snapshot: ActivitySnapshot,
    selected_tab: ActivityTab,
    selected_pid: Option<i32>,
    mounted: bool,
}

impl Default for ActivityMonitorApp {
    fn default() -> Self {
        Self::from_snapshot(ActivitySnapshot::placeholder())
    }
}

impl ActivityMonitorApp {
    pub fn from_snapshot(snapshot: ActivitySnapshot) -> Self {
        let selected_pid = snapshot.processes.first().map(|process| process.pid);
        Self {
            snapshot,
            selected_tab: ActivityTab::Cpu,
            selected_pid,
            mounted: false,
        }
    }

    fn sorted_processes(&self) -> Vec<ProcessSample> {
        self.snapshot.processes_for(self.selected_tab)
    }

    fn selected_process(&self) -> Option<ProcessSample> {
        self.snapshot
            .selected_process(self.selected_tab, self.selected_pid)
    }

    fn table_alignments(&self) -> Vec<TableAlignment> {
        match self.selected_tab {
            ActivityTab::Cpu => vec![
                TableAlignment::Left,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Left,
                TableAlignment::Right,
                TableAlignment::Left,
            ],
            ActivityTab::Memory => vec![
                TableAlignment::Left,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Left,
            ],
            ActivityTab::Energy => vec![
                TableAlignment::Left,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Left,
            ],
            ActivityTab::Disk => vec![
                TableAlignment::Left,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Left,
            ],
            ActivityTab::Network => vec![
                TableAlignment::Left,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Right,
                TableAlignment::Left,
            ],
        }
    }

    fn status_text(&self) -> String {
        let suffix = self
            .snapshot
            .warning
            .as_deref()
            .map(|warning| format!(" · {warning}"))
            .unwrap_or_default();
        format!("> activity monitor · all processes{suffix}")
    }

    fn selection_text(&self) -> String {
        match self.selected_process() {
            Some(process) => format!(
                "Selected process: {} · pid: {} · user: {} · Tab: {}",
                process.name,
                process.pid,
                process.user,
                self.selected_tab.title()
            ),
            None => format!(
                "Selected process: none · Tab: {}",
                self.selected_tab.title()
            ),
        }
    }

    fn footer_blocks(&self) -> Vec<Element<ActivityMonitorMessage>> {
        let selected = self.selected_process();
        let cpu = Block::bordered()
            .title("CPU")
            .child(
                Paragraph::new(format!(
                    "System: {:.1}%\nUser: {:.1}%\nIdle: {:.1}%",
                    self.snapshot.cpu.system_percent,
                    self.snapshot.cpu.user_percent,
                    self.snapshot.cpu.idle_percent,
                ))
                .build(),
            )
            .build();

        let memory = Block::bordered()
            .title("Memory")
            .child(
                Paragraph::new(format!(
                    "Used: {}\nApp: {}\nSwap: {}",
                    format_bytes(self.snapshot.memory.used_bytes),
                    format_bytes(self.snapshot.memory.app_bytes),
                    format_bytes(self.snapshot.memory.swap_used_bytes),
                ))
                .build(),
            )
            .build();

        let activity = Block::bordered()
            .title(match self.selected_tab {
                ActivityTab::Cpu => "CPU details",
                ActivityTab::Memory => "Memory details",
                ActivityTab::Energy => "Energy details",
                ActivityTab::Disk => "Disk details",
                ActivityTab::Network => "Network details",
            })
            .child(Paragraph::new(self.detail_text(selected)).build())
            .build();

        vec![cpu, memory, activity]
    }

    fn detail_text(&self, selected: Option<ProcessSample>) -> String {
        match (self.selected_tab, selected) {
            (_, None) => "No process selected".to_string(),
            (ActivityTab::Cpu, Some(process)) => format!(
                "Threads: {}\nIdle wake ups: {}\nGPU: {:.1}%",
                process.threads, process.idle_wakeups, process.gpu_percent
            ),
            (ActivityTab::Memory, Some(process)) => format!(
                "Memory: {}\nCompressed: {}\nKind: {}",
                format_bytes(process.memory_bytes),
                format_bytes(process.compressed_bytes),
                process.kind
            ),
            (ActivityTab::Energy, Some(process)) => format!(
                "Impact: {:.1}\n12 hr avg: {:.1}\nCPU: {:.1}%",
                process.energy_impact, process.energy_impact_avg, process.cpu_percent
            ),
            (ActivityTab::Disk, Some(process)) => format!(
                "Read: {}\nWrite: {}\nSystem: {}/s write",
                format_bytes(process.disk_read_bytes),
                format_bytes(process.disk_write_bytes),
                format_bytes(self.snapshot.disk.write_per_sec)
            ),
            (ActivityTab::Network, Some(process)) => format!(
                "In: {}\nOut: {}\nSystem recv: {}/s",
                format_bytes(process.network_in_bytes),
                format_bytes(process.network_out_bytes),
                format_bytes(self.snapshot.network.recv_per_sec)
            ),
        }
    }

    fn apply_snapshot(&mut self, snapshot: ActivitySnapshot) {
        let current_selected = self.selected_pid;
        self.snapshot = snapshot;
        self.selected_pid = current_selected
            .filter(|pid| {
                self.snapshot
                    .processes
                    .iter()
                    .any(|process| process.pid == *pid)
            })
            .or_else(|| self.snapshot.processes.first().map(|process| process.pid));
    }
}

impl App for ActivityMonitorApp {
    type Message = ActivityMonitorMessage;

    fn mount(&mut self, handle: &RuntimeHandle<Self::Message>) {
        if self.mounted {
            return;
        }

        self.mounted = true;
        let handle = handle.clone();
        let emitter = handle.clone();
        handle.spawn(async move {
            let mut sampler = ActivitySampler::new();
            loop {
                let message = match sampler.collect() {
                    Ok(snapshot) => ActivityMonitorMessage::Snapshot(snapshot),
                    Err(error) => ActivityMonitorMessage::SamplingFailed(error),
                };

                if emitter.emit(message).is_err() {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let processes = self.sorted_processes();
        let selected_index = self
            .selected_pid
            .and_then(|pid| processes.iter().position(|process| process.pid == pid))
            .or_else(|| (!processes.is_empty()).then_some(0));

        let tabs = Tabs::new(ActivitySnapshot::tab_titles())
            .selected(Some(self.selected_tab.index()))
            .on_select(|index| Some(ActivityMonitorMessage::SelectTab(index)))
            .build();

        let header = self.snapshot.headers_for(self.selected_tab);
        let rows = self.snapshot.rows_for(self.selected_tab);
        let widths = vec![Constraint::Fill(1); header.len().max(1)];
        let table = Table::new(rows, widths)
            .header(header)
            .alignments(self.table_alignments())
            .selected(selected_index)
            .on_select(|index| Some(ActivityMonitorMessage::SelectProcess(index)))
            .layout(Layout {
                width: Length::Fill,
                height: Length::Fill,
            })
            .build();

        let body = Block::bordered()
            .title("All Processes")
            .layout(Layout {
                width: Length::Fill,
                height: Length::Fill,
            })
            .child(table)
            .build();

        let footer = Box::column()
            .gap(1)
            .child(Text::new(self.selection_text()).build())
            .child(
                Box::row()
                    .gap(1)
                    .layout(Layout {
                        width: Length::Fill,
                        height: Length::Fixed(5),
                    })
                    .children(self.footer_blocks())
                    .build(),
            )
            .build();

        Shell::new()
            .header(
                Box::column()
                    .gap(1)
                    .child(StatusBar::new(self.status_text()).build())
                    .child(tabs)
                    .build(),
            )
            .body(body)
            .footer(footer)
            .build()
    }

    fn update(&mut self, message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {
        match message {
            ActivityMonitorMessage::Snapshot(snapshot) => self.apply_snapshot(snapshot),
            ActivityMonitorMessage::SelectTab(index) => {
                self.selected_tab = ActivityTab::from_index(index);
                self.selected_pid = self
                    .snapshot
                    .processes_for(self.selected_tab)
                    .first()
                    .map(|process| process.pid);
            }
            ActivityMonitorMessage::SelectProcess(index) => {
                if let Some(process) = self.sorted_processes().get(index) {
                    self.selected_pid = Some(process.pid);
                }
            }
            ActivityMonitorMessage::SamplingFailed(error) => {
                self.snapshot.warning = Some(error);
            }
        }
    }
}
