use miku_macros::{define_device, rpc};
use miniserde::Deserialize;

type Void = Option<()>;

/// An opencomputers HLApi device.
pub trait RPCDevice {
    /// Returns uuid of this device.
    fn id(&self) -> &str;

    /// Create a device wrapper from an id.
    fn from_id(id: String) -> Self;
}

/// A HLApi device that has an identity - like "redstone". This is used for the [crate::bus::DeviceBus::wrap] method.
pub trait IdentifiedDevice: RPCDevice {
    const IDENTITY: &'static str;
}

pub trait EnergyStorage: RPCDevice {
    rpc!(get_energy_stored -> i64; "getEnergyStored");
    rpc!(get_max_energy_stored -> i64; "getMaxEnergyStored");
    rpc!(can_extract_energy -> bool; "canExtractEnergy");
    rpc!(can_receive_energy -> bool; "canReceiveEnergy");
}

pub trait ItemHandler: RPCDevice {
    rpc!(get_item_slot_counts -> i64; "getItemSlotCount");
    rpc!(get_item_slot_limit -> i64; "getItemSlotLimit"; slot: i64);
    rpc!(get_item_stack_in_slot<T: Deserialize> -> T; "getItemStackInSlot"; slot: i64);
}

pub trait FluidHandler: RPCDevice {
    rpc!(get_fluid_tanks -> i64; "getFluidTanks");
    rpc!(get_fluid_tank_capacity -> i64; "getFluidTankCapacity"; tank: i64);
    rpc!(get_fluid_in_tank<T: Deserialize> -> T; "getFluidInTank"; tank: i64);
}

pub trait RedstoneInterface: RPCDevice {
    rpc!(get_redstone_input -> i64; "getRedstoneInput"; side: &str);
    rpc!(get_redstone_output -> i64; "getRedstoneOutput"; side: &str);
    rpc!(set_redstone_output -> Void; "setRedstoneOutput"; side: &str, val: i64);
}

pub trait SoundInterface: RPCDevice {
    rpc!(find_sound -> Vec<String>; "findSound"; name: &str);
    rpc!(play_sound -> Void; "playSound"; name: &str);
}

define_device!(RedstoneCard, "redstone", [RedstoneInterface]);
define_device!(SoundCard, "sound", [SoundInterface]);
