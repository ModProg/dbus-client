#![allow(non_snake_case, non_camel_case_types)]
use std::time::Duration;

use dbus_client::{dbus_object, Append, Arg, DbusObject, Get};

dbus_object! {
    /// Interface implemented by the main wpa_supplicant D-Bus object registered in the bus with fi.w1.wpa_supplicant1 name.
    WpaSupplicant("fi.w1.wpa_supplicant1", "/fi/w1/wpa_supplicant1", system)
    "fi.w1.wpa_supplicant1" {
        /// Registers a wireless interface in `wpa_supplicant`.
        /// # Errors
        /// - `fi.w1.wpa_supplicant1.InterfaceExists`
        ///   `wpa_supplicant` already controls this interface.
        /// - `fi.w1.wpa_supplicant1.UnknownError`
        ///   Creating interface failed for an unknown reason.
        /// - `fi.w1.wpa_supplicant1.InvalidArgs`
        ///   Invalid entries were found in the passed argument.
        CreateInterface(args: CreateInterface) -> @Interface;
        /// Deregisters a wireless interface from `wpa_supplicant`.
        ///
        /// # Arguments
        /// - `o` A D-Bus path to an object representing an interface
        ///   to remove returned by [`CreateInterface`](WpaSupplicant::CreateInterface).
        ///
        /// # Errors
        /// - `fi.w1.wpa_supplicant1.InterfaceUnknown`
        ///   Object pointed by the path doesn't exist or doesn't represent an interface.
        /// - `fi.w1.wpa_supplicant1.UnknownError`
        ///   Removing interface failed for an unknown reason.
        RemoveInterface(o: Interface);
        /// Returns a D-Bus path to an object related to an interface which `wpa_supplicant` already controls.
        ///
        /// # Arguments
        /// - `ifname` Name of the network interface, e.g., `wlan0`
        ///
        /// # Errors
        /// - `fi.w1.wpa_supplicant1.InterfaceUnknown`
        ///   An interface with the passed name in not controlled by `wpa_supplicant`.
        /// - `fi.w1.wpa_supplicant1.UnknownError`
        ///   Getting an interface object path failed for an unknown reason.
        GetInterface(ifname: s) -> @Interface;
        /// Global `wpa_supplicant` debugging level.
        mut DebugLevel: DebugLevel;
        /// Global `wpa_supplicant` debugging parameter. Determines if timestamps are shown in debug logs.
        mut DebugTimestamp: b;
        /// Global `wpa_supplicant` debugging parameter. Determines if secrets are shown in debug logs.
        mut DebugShowKeys: b;
        /// An array with paths to D-Bus objects representing controlled interfaces each.
        Interfaces: a @Interface;
        /// An array with supported EAP methods names.
        EapMethods: a s;
        /// An array with supported capabilities (e.g., "ap", "ibss-rsn", "p2p", "interworking").
        Capabilities: a s;
        /// Wi-Fi Display subelements.
        mut WFDIEs: a y;
        // TODO signals
        // /// A new interface was added to wpa_supplicant.
        // ///
        // /// # Arguments
        // /// - `interface` A D-Bus path to an object representing the added interface
        // /// - `properties` A dictionary containing properties of added interface.
        // ~InterfaceAdded(interface: @Interface, properties: a{s v});
        // /// An interface was removed from wpa_supplicant.
        // ///
        // /// # Arguments
        // /// - `interface` A D-Bus path to an object representing the removed interface.
        // ~InterfaceRemoved(interface: @Interface)
        // /// Some properties have changed.
        // ///
        // /// # Arguments
        // /// - `properties` A dictionary with pairs of properties names which have changed and theirs new values. Possible dictionary keys are: "DebugParams"
        // ~PropertiesChanged(properties: a{s v})
    }
}

#[derive(Append)]
/// A dictionary with arguments used to add the interface to `wpa_supplicant`.
struct CreateInterface {
    /// Name of the network interface to control, e.g., `wlan0`
    Ifname: String,
    /// Name of the bridge interface to control, e.g., `br0`
    BridgeIfName: Option<String>,
    /// Driver name which the interface uses, e.g., `nl80211`
    Driver: Option<String>,
    /// Configuration file path
    ConfigFile: Option<String>,
}

#[derive(Arg, Get, Append, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DebugLevel {
    msgdump,
    debug,
    info,
    warning,
    error,
}

dbus_object! {
    /// Interface implemented by objects related to network interface added to `wpa_supplicant`, i.e., returned [`WpaSupplicant::CreateInterface`].
    Interface "fi.w1.wpa_supplicant1.Interface" {
        /// Triggers a scan.
        ///
        /// # Errors
        /// - `fi.w1.wpa_supplicant1.InvalidArgs` Invalid entries were found in the passed argument.
        Scan(args: InterfaceScan);
        /// Disassociates the interface from current network.
        ///
        /// # Errors
        /// - `fi.w1.wpa_supplicant1.NotConnected` Interface is not connected to any network.
        Disconnect();
        /// Adds a new network to the interface.
        /// # Arguments
        /// - `args` A dictionary with network configuration. Dictionary entries are equivalent to entries in the "network" block in `wpa_supplicant` configuration file. Entry values should be appropriate type to the entry, e.g., an entry with key "frequency" should have value type int.
        /// # Errors
        /// - `fi.w1.wpa_supplicant1.InvalidArgs` Invalid entries were found in the passed argument.
        /// - `fi.w1.wpa_supplicant1.UnknownError` Adding network failed for an unknown reason.
        AddNetwork(args: a{s v}) -> @Network;
        // TODO there is so much :o
    }
}

#[derive(Append)]
struct InterfaceScan {
    /// Type of the scan.
    Type: InterfaceScanType,
    /// Array of SSIDs to scan for (applies only if scan type is active)
    SSIDs: Option<Vec<Vec<u8>>>,
    /// Information elements to used in active scan (applies only if scan type
    /// is active)
    IEs: Option<Vec<Vec<u8>>>,
    /// Array of frequencies to scan in form of (center, width) in MHz.
    Channels: Option<Vec<(u32, u32)>>,
    /// TRUE (or absent) to allow a roaming decision based on the results of
    /// this scan, FALSE to prevent a roaming decision.
    AllowRoam: Option<bool>,
}

#[derive(Arg, Get, Append, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum InterfaceScanType {
    active,
    passive,
}

dbus_object!(Network);

type Result<T = (), E = dbus::Error> = std::result::Result<T, E>;

fn main() -> Result {
    let wpa = WpaSupplicant::system(Duration::from_secs(1))?;

    dbg!(wpa.get_DebugLevel()?);

    dbg!(wpa.CreateInterface(CreateInterface {
        Ifname: "name".into(),
        BridgeIfName: None,
        Driver: None,
        ConfigFile: None,
    }))?;

    for interface in wpa.get_Interfaces()? {
        eprintln!("{interface:?}")
    }

    Ok(())
}
