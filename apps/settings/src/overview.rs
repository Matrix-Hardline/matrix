//! Overview — the dashboard of Matrix Einstellungen (section 0, start page).
//!
//! Taken empirically from the reference model (System Settings → General →
//! About + Storage): hero with device name and a quiet subtitle, below it
//! an info card with label-left/value-right rows, plus live cards
//! (processor, memory, storage with segment bar, network, energy).
//! The live values run via the host tick (3 s) — no own subscription needed.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, row, Space};
use matrixkit_widgets::iced::{Element, Length, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::time::Instant;
use sysinfo::{Disks, System};

pub struct Panel {
    pub palette: mk::Palette,
    pub(crate) sys: System,
    pub(crate) net: sysinfo::Networks,
    pub(crate) net_state: Instant,
    // Static identity (gathered once at start)
    pub(crate) device: String,
    pub(crate) base: String,
    pub(crate) kernel: String,
    pub(crate) hostname: String,
    pub(crate) cpu_model: String,
    pub(crate) cpu_cores: usize,
    // Live values
    pub(crate) cpu: f32,
    pub(crate) temperature: Option<f32>,
    pub(crate) mem_used: u64,
    pub(crate) mem_total: u64,
    pub(crate) disk_used: u64,
    pub(crate) disk_total: u64,
    pub(crate) battery: Option<(u8, bool)>,
    pub(crate) ip: String,
    pub(crate) rx_rate: f64,
    pub(crate) tx_rate: f64,
    /// Round 98: the biggest Matrix parts (RAM + GPU memory), refreshed
    /// with the live values.
    pub(crate) apps: Vec<mk::memory::AppUse>,
}

#[derive(Debug, Clone)]
pub enum Msg {}

/// First line of a file, trimmed.
fn line_from(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
}

/// PRETTY_NAME from os-release — the base beneath Matrix.
fn os_base() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| String::from("Linux"))
}

/// The processor's temperature (the kit's choice of zones, round 32).
fn temperature_read() -> Option<f32> {
    mk::device::cpu_temperature()
}

/// Battery: (% level, charging?) — round 178: the kit's gauge rule (the Duo's «battery» was never shown here).
fn battery_read() -> Option<(u8, bool)> {
    mk::power::battery().map(|b| (b.percent.min(100) as u8, b.charging))
}

/// First global IPv4 address (without the interface zoo).
fn ip_read() -> String {
    // Round 56: from the kernel (getifaddrs) — was `ip` every 3 s.
    mk::device::ipv4_global().unwrap_or_else(|| String::from("—"))
}

// R65b: the number language lives in mk::format (Foundation extract).

impl Panel {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let cpu_model = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| String::from("Prozessor"));
        let mut panel = Self {
            palette: mk::Palette::load().unwrap_or_default(),
            net: sysinfo::Networks::new_with_refreshed_list(),
            net_state: Instant::now(),
            device: line_from("/sys/class/dmi/id/product_name")
                .unwrap_or_else(|| String::from("Dieser Rechner")),
            base: os_base(),
            kernel: line_from("/proc/sys/kernel/osrelease").unwrap_or_default(),
            hostname: System::host_name().unwrap_or_else(|| String::from("matrix")),
            cpu_cores: sys.cpus().len(),
            cpu_model,
            cpu: 0.0,
            temperature: None,
            mem_used: 0,
            mem_total: 0,
            disk_used: 0,
            disk_total: 0,
            battery: None,
            ip: ip_read(),
            rx_rate: 0.0,
            tx_rate: 0.0,
            apps: Vec::new(),
            sys,
        };
        panel.measure();
        panel
    }

    /// From the host per tick (3 s): palette follows, refresh metrics.
    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
        self.measure();
    }

    pub(crate) fn measure(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.cpu = self.sys.global_cpu_usage();
        self.mem_used = self.sys.used_memory();
        self.mem_total = self.sys.total_memory();
        self.apps = mk::memory::app_usage();
        self.temperature = temperature_read();
        self.battery = battery_read();
        self.ip = ip_read();

        // On bootc/ostree "/" is an overlay — find the real disk via the
        // known mounts, else take the largest (like sysmon).
        let disks = Disks::new_with_refreshed_list();
        let by_mount =
            |m: &str| disks.list().iter().find(|d| d.mount_point().to_str() == Some(m));
        if let Some(d) = by_mount("/sysroot")
            .or_else(|| by_mount("/var/home"))
            .or_else(|| by_mount("/var"))
            .or_else(|| by_mount("/"))
            .filter(|d| d.total_space() > 2_000_000_000)
            .or_else(|| disks.list().iter().max_by_key(|d| d.total_space()))
        {
            self.disk_total = d.total_space();
            self.disk_used = d.total_space() - d.available_space();
        }

        // Network rates via the receive/send delta since the last tick.
        let dt = self.net_state.elapsed().as_secs_f64().max(0.5);
        self.net.refresh();
        self.net_state = Instant::now();
        let (mut rx, mut tx) = (0u64, 0u64);
        for (name, data) in self.net.iter() {
            // Loopback does not count — else PipeWire "radios" into the dashboard.
            if name == "lo" {
                continue;
            }
            rx += data.received();
            tx += data.transmitted();
        }
        self.rx_rate = rx as f64 / dt;
        self.tx_rate = tx as f64 / dt;
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {}
    }

    pub fn footer_text(&self) -> String {
        format!(
            "{} · Kernel {} · seit {}",
            self.base,
            self.kernel,
            mk::format::duration(System::uptime())
        )
    }

    /// `width`: the window width — the families pick the form (R207).
    pub fn view(&self, width: f32) -> Element<'_, Msg> {
        let p = self.palette;

        // --- Hero: like the reference About — device name large, quiet row below ---
        let hero = column![
            mkw::txt(String::from("Matrix"), mk::typo::LARGE_TITLE, p.on_surface),
            Space::new().height(2),
            mkw::txt(self.device.clone(), mk::typo::BODY, p.on_surface_variant),
            Space::new().height(2),
            // The house signature — like the reference "M1, 2020" row,
            // except the creator stands here.
            mkw::txt(
                String::from("Matrix Hardline \u{00b7} matrixhardline@ik.me"),
                mk::typo::LABEL,
                p.primary,
            ),
        ]
        .align_x(iced::Alignment::Center);
        let hero = container(hero).width(Length::Fill).align_x(iced::Alignment::Center);

        // --- Info card (reference: label left in grey, value right) ---
        let value_row = |title: &'static str, value: String| mkw::row_value(title, None, value, p);
        let info = mkw::section(
            "",
            vec![
                value_row("Gerät", self.device.clone()),
                value_row(
                    "Betriebssystem",
                    // The image sets PRETTY_NAME itself to Matrix —
                    // then do not double it (screenshot finding R51).
                    if self.base.starts_with("Matrix") {
                        self.base.clone()
                    } else {
                        format!("Matrix \u{00b7} {}", self.base)
                    },
                ),
                value_row("Kernel", self.kernel.clone()),
                value_row("Rechnername", self.hostname.clone()),
                value_row("Betriebszeit", mk::format::duration(System::uptime())),
            ],
            p,
        );

        // --- Live cards ---
        let cpu_value = format!("{:.0} %", self.cpu);
        let cpu_desc = match self.temperature {
            Some(t) => format!("{} · {} Kerne · {:.0} °C", self.cpu_model, self.cpu_cores, t),
            None => format!("{} · {} Kerne", self.cpu_model, self.cpu_cores),
        };
        let processor = self.metric_card(
            "PROZESSOR",
            mkw::symbol::MEMORY,
            cpu_value,
            Some(cpu_desc),
            self.cpu / 100.0,
            p,
        );

        let ram_portion = if self.mem_total > 0 {
            self.mem_used as f32 / self.mem_total as f32
        } else {
            0.0
        };
        let ram = self.metric_card(
            "ARBEITSSPEICHER",
            mkw::symbol::MONITORING,
            mk::format::bytes_decimal_pair(self.mem_used, self.mem_total),
            None,
            ram_portion,
            p,
        );

        let disk_portion = if self.disk_total > 0 {
            self.disk_used as f32 / self.disk_total as f32
        } else {
            0.0
        };
        let memory = self.metric_card(
            "SPEICHER",
            mkw::symbol::STORAGE,
            // Disks DECIMAL like the reference file manager (R65b) — was binary.
            format!("{} verwendet", mk::format::bytes_paar(self.disk_used, self.disk_total)),
            None,
            disk_portion,
            p,
        );

        let network = mkw::section(
            "NETZWERK",
            vec![mkw::row_item(
                "IP-Adresse",
                Some(self.ip.as_str()),
                Some(symbol_circle(mkw::symbol::PUBLIC, p)),
                Some(
                    mkw::txt(
                        format!("↓ {}  ↑ {}", mk::format::rate(self.rx_rate), mk::format::rate(self.tx_rate)),
                        mk::typo::BODY,
                        p.on_surface_variant,
                    )
                    .into(),
                ),
                p,
            )],
            p,
        );

        let mut column = column![
            Space::new().height(mk::spacing::M),
            hero,
            Space::new().height(mk::spacing::L),
            info,
            Space::new().height(mk::spacing::L),
            mkw::ui::card_row(vec![processor, ram], width, 260.0),
            Space::new().height(mk::spacing::L),
            memory,
            Space::new().height(mk::spacing::L),
            network,
        ]
        .spacing(0);

        // Energy card only if a battery exists (the PC has none).
        if let Some((percent, loading)) = self.battery {
            let symbol = if loading {
                mkw::symbol::BATTERY_CHARGING
            } else {
                mkw::symbol::BATTERY_FULL
            };
            let status = if loading { "Lädt / am Netz" } else { "Entlädt" };
            column = column
                .push(Space::new().height(mk::spacing::L))
                .push(self.metric_card(
                    "ENERGIE",
                    symbol,
                    format!("{percent} %"),
                    Some(status.to_string()),
                    percent as f32 / 100.0,
                    p,
                ));
        }

        column.into()
    }

    /// A live card: symbol in a circle, value right, bar below.
    fn metric_card(
        &self,
        title: &'static str,
        symbol: char,
        value: String,
        description: Option<String>,
        portion: f32,
        p: mk::Palette,
    ) -> Element<'_, Msg> {
        let mut content = column![
            row![
                symbol_circle(symbol, p),
                Space::new().width(mk::spacing::S),
                mkw::txt(value, mk::typo::BODY, p.on_surface),
                Space::new().width(Length::Fill),
            ]
            .align_y(iced::Alignment::Center),
        ]
        .spacing(mk::spacing::XS);
        if let Some(b) = description {
            content = content.push(mkw::txt(b, mk::typo::LABEL, p.on_surface_variant));
        }
        content = content.push(Space::new().height(2)).push(meter(portion, p));
        let card: Element<'_, Msg> =
            container(content).padding(mk::spacing::M).width(Length::Fill).into();
        mkw::section(title, vec![card], p)
    }
}

/// Symbol in a quiet primary-container circle — the card signature.
fn symbol_circle<'a, M: 'a>(glyph: char, p: mk::Palette) -> Element<'a, M> {
    container(mkw::symbol(glyph, mk::icon_size::NORMAL, p.primary))
        .padding(6)
        .style(move |_| container::Style {
            background: Some(color(p.primary_container).into()),
            border: iced::Border { radius: mk::radius::capsule(36.0).into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

/// Pill bar like in sysmon: track + primary fill.
fn meter<'a, M: 'a>(portion: f32, p: mk::Palette) -> Element<'a, M> {
    let h = 6.0;
    let full = ((portion.clamp(0.0, 1.0) * 1000.0).round() as u16).clamp(1, 999);
    let rest = 1000 - full;
    let fill = if portion >= 0.9 { p.error } else { p.primary };
    let track = p.on_surface.over(p.surface_container_high, 0.10);
    container(row![
        container(Space::new().width(Length::Fill).height(Length::Fixed(h)))
            .width(Length::FillPortion(full))
            .style(move |_| container::Style {
                background: Some(color(fill).into()),
                border: iced::Border { radius: (h / 2.0).into(), ..Default::default() },
                ..Default::default()
            }),
        Space::new().width(Length::FillPortion(rest)).height(Length::Fixed(h)),
    ])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(color(track).into()),
        border: iced::Border { radius: (h / 2.0).into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_levels() {
        assert_eq!(mk::format::duration(300), "5 Min.");
        assert_eq!(mk::format::duration(3_900), "1 Std. 5 Min.");
        assert_eq!(mk::format::duration(90_000), "1 T. 1 Std.");
    }

    #[test]
    fn ram_binary_disk_decimal() {
        assert_eq!(mk::format::bytes_decimal(8 * 1_073_741_824), "8.0 GB");
        assert_eq!(mk::format::bytes(512_000_000_000), "512 GB");
    }

    #[test]
    fn rate_units() {
        assert_eq!(mk::format::rate(2_100_000.0), "2.1 MB/s");
    }
}
