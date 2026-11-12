use std::time::{Duration, Instant};

use pi_dashboard_core::{
    bluetooth::BluetoothService, computer::ComputerService, spotify::SpotifyService,
    system_info::SystemInfoService, volume::VolumeService,
};

use crate::snapshots::{
    BluetoothDeviceSnapshot, BluetoothStatus, ComputerStatus, DashboardStatus, SpotifyStatus,
    SystemSnapshot, VolumeStatus,
};

#[derive(Default)]
pub(crate) struct DashboardState {
    system: SystemInfoService,
    pub(crate) volume: VolumeService,
    pub(crate) bluetooth: BluetoothService,
    computer: ComputerService,
    spotify: SpotifyService,
    spotify_last_refresh: Option<Instant>,
}

impl DashboardState {
    pub(crate) fn poll(&mut self) {
        self.system.poll();
        self.system.ensure_fresh();
        self.volume.poll();
        self.volume.ensure_initialized();
        self.bluetooth.poll();
        self.bluetooth.ensure_initialized();
        self.computer.poll();
        self.spotify.poll();
        if self
            .spotify_last_refresh
            .is_none_or(|last| last.elapsed() >= Duration::from_secs(5))
        {
            self.refresh_spotify();
        }
    }

    pub(crate) fn refresh_spotify(&mut self) {
        self.spotify.poll();
        self.spotify.refresh();
        self.spotify_last_refresh = Some(Instant::now());
    }

    pub(crate) fn status(&self) -> DashboardStatus {
        let message = self.volume.message();
        DashboardStatus {
            system: self.system.snapshot().map(SystemSnapshot::from),
            system_loading: self.system.snapshot().is_none() && self.system.error().is_none(),
            system_error: self.system.error().map(str::to_owned),
            bluetooth: BluetoothStatus {
                devices: self
                    .bluetooth
                    .devices()
                    .iter()
                    .map(|device| BluetoothDeviceSnapshot {
                        address: device.address.clone(),
                        name: device.name.clone(),
                        paired: device.paired,
                        trusted: device.trusted,
                        connected: device.connected,
                    })
                    .collect(),
                selected_address: self
                    .bluetooth
                    .selected_device()
                    .map(|device| device.address.clone()),
                busy: self.bluetooth.is_busy(),
                busy_label: self.bluetooth.busy_label().map(str::to_owned),
                message: self
                    .bluetooth
                    .message()
                    .filter(|(_, error)| !error)
                    .map(|(text, _)| text.to_owned()),
                error: self
                    .bluetooth
                    .message()
                    .filter(|(_, error)| *error)
                    .map(|(text, _)| text.to_owned()),
            },
            computer: ComputerStatus {
                url: self.computer.url().to_owned(),
                connected: self.computer.is_connected(),
                connecting: self.computer.is_connecting(),
                fields: self
                    .computer
                    .snapshot()
                    .map(|snapshot| snapshot.fields.clone()),
                error: self.computer.error().map(str::to_owned),
            },
            spotify: SpotifyStatus {
                running: self.spotify.is_running(),
            },
            volume: VolumeStatus {
                level: self.volume.level(),
                muted: self.volume.level().map(|_| self.volume.is_muted()),
                backend: self.volume.backend_name().map(str::to_owned),
                busy: self.volume.is_busy(),
                busy_label: self.volume.busy_label().map(str::to_owned),
                message: message
                    .filter(|(_, error)| !error)
                    .map(|(text, _)| text.to_owned()),
                error: message
                    .filter(|(_, error)| *error)
                    .map(|(text, _)| text.to_owned()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_status_does_not_invent_readings() {
        let status = DashboardState::default().status();
        assert!(status.system.is_none());
        assert!(status.system_loading);
        assert!(status.volume.level.is_none());
        assert!(status.volume.muted.is_none());
        assert!(!status.volume.busy);
        assert!(status.bluetooth.devices.is_empty());
        assert!(status.bluetooth.selected_address.is_none());
        assert!(!status.bluetooth.busy);
        assert!(status.computer.fields.is_none());
        assert!(!status.computer.connected);
        assert!(status.computer.connecting);
        assert!(!status.spotify.running);
    }
}
