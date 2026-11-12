use pi_dashboard_core::bluetooth::{BluetoothService, DeviceAction};

use std::sync::Mutex;

use crate::{snapshots::DashboardStatus, state::DashboardState};

#[derive(Debug, PartialEq)]
enum BluetoothAction {
    Refresh,
    Scan,
    Select,
    Pair,
    Trust,
    Connect,
    Disconnect,
    Remove,
}

impl BluetoothAction {
    fn parse(action: &str) -> Result<Self, String> {
        match action {
            "refresh" => Ok(Self::Refresh),
            "scan" => Ok(Self::Scan),
            "select" => Ok(Self::Select),
            "pair" => Ok(Self::Pair),
            "trust" => Ok(Self::Trust),
            "connect" => Ok(Self::Connect),
            "disconnect" => Ok(Self::Disconnect),
            "remove" => Ok(Self::Remove),
            _ => Err("Invalid Bluetooth action. Allowed: refresh, scan, select, pair, trust, connect, disconnect, remove.".to_owned()),
        }
    }
}

fn validate_address(address: &str) -> Result<(), String> {
    let bytes = address.as_bytes();
    if bytes.len() == 17
        && bytes.iter().enumerate().all(|(index, byte)| {
            if index % 3 == 2 {
                *byte == b':'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
    {
        Ok(())
    } else {
        Err("Invalid Bluetooth address. Expected six colon-separated hexadecimal pairs.".to_owned())
    }
}

fn apply_bluetooth_action(
    service: &mut BluetoothService,
    action: BluetoothAction,
    address: Option<&str>,
) -> Result<(), String> {
    let device_action = match action {
        BluetoothAction::Pair => Some(DeviceAction::Pair),
        BluetoothAction::Trust => Some(DeviceAction::Trust),
        BluetoothAction::Connect => Some(DeviceAction::Connect),
        BluetoothAction::Disconnect => Some(DeviceAction::Disconnect),
        BluetoothAction::Remove => Some(DeviceAction::Remove),
        _ => None,
    };
    let requires_address = device_action.is_some() || action == BluetoothAction::Select;
    if requires_address {
        validate_address(address.ok_or("A Bluetooth device address is required.")?)?;
    } else if address.is_some() {
        return Err("Refresh and scan do not accept a device address.".to_owned());
    }
    service.poll();
    if service.is_busy() {
        return Err("Bluetooth is busy. Wait for the current operation to finish.".to_owned());
    }
    if requires_address {
        let address = address.expect("validated address");
        let known_address = service
            .devices()
            .iter()
            .find(|device| device.address.eq_ignore_ascii_case(address))
            .map(|device| device.address.clone())
            .ok_or("Bluetooth device is no longer available. Refresh or scan first.")?;
        if !service.select_device(&known_address) {
            return Err("Bluetooth device is no longer available.".to_owned());
        }
    }
    match action {
        BluetoothAction::Refresh => service.refresh(),
        BluetoothAction::Scan => service.scan(),
        BluetoothAction::Select => {}
        _ => service.perform(device_action.expect("allowlisted device action")),
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn bluetooth_action(
    action: String,
    address: Option<String>,
    state: tauri::State<'_, Mutex<DashboardState>>,
) -> Result<DashboardStatus, String> {
    let action = BluetoothAction::parse(&action)?;
    let mut state = state
        .lock()
        .map_err(|_| "Dashboard state is unavailable.".to_owned())?;
    apply_bluetooth_action(&mut state.bluetooth, action, address.as_deref())?;
    Ok(state.status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bluetooth_actions_are_an_exact_allowlist() {
        for action in [
            "refresh",
            "scan",
            "select",
            "pair",
            "trust",
            "connect",
            "disconnect",
            "remove",
        ] {
            assert!(BluetoothAction::parse(action).is_ok());
        }
        for action in [
            "",
            "REFRESH",
            " refresh",
            "shell",
            "connect; reboot",
            "start",
            "stop",
        ] {
            assert!(BluetoothAction::parse(action).is_err());
        }
    }

    #[test]
    fn bluetooth_addresses_and_targets_are_validated_before_work() {
        for address in ["AA:BB:CC:DD:EE:FF", "aa:bb:01:23:45:ff"] {
            assert!(validate_address(address).is_ok());
        }
        for address in [
            "",
            "AA:BB:CC:DD:EE",
            "AA:BB:CC:DD:EE:GG",
            " AA:BB:CC:DD:EE:FF",
            "AA:BB:CC:DD:EE:FF; reboot",
            "éé:BB:CC:DD:EE:FF",
        ] {
            assert!(validate_address(address).is_err());
        }
        let mut service = BluetoothService::default();
        for action in [
            BluetoothAction::Select,
            BluetoothAction::Pair,
            BluetoothAction::Trust,
            BluetoothAction::Connect,
            BluetoothAction::Disconnect,
            BluetoothAction::Remove,
        ] {
            assert!(
                apply_bluetooth_action(&mut service, action, Some("AA:BB:CC:DD:EE:FF")).is_err()
            );
            assert!(!service.is_busy());
        }
        assert!(apply_bluetooth_action(&mut service, BluetoothAction::Pair, None).is_err());
        assert!(
            apply_bluetooth_action(
                &mut service,
                BluetoothAction::Scan,
                Some("AA:BB:CC:DD:EE:FF")
            )
            .is_err()
        );
        assert!(!service.is_busy());
    }
}
