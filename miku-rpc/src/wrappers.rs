use crate::types::ImportFileInfo;
use miku_macros::{define_device, rpc};
use miniserde::Deserialize;

/// An opencomputers HLApi device.
pub trait RPCDevice {
    /// Returns uuid of this device.
    fn id(&self) -> &str;

    /// Create a device wrapper from an id.
    fn from_id(id: String) -> Self;
}

/// A HLApi device that has an identity - like "redstone". This is used for the [crate::DeviceBus::wrap] method.
pub trait IdentifiedDevice: RPCDevice {
    const IDENTITY: &'static str;
}

pub trait EnergyStorage: RPCDevice {
    #[rpc("getEnergyStored")]
    fn get_energy_stored() -> i64;

    #[rpc("getMaxEnergyStored")]
    fn get_max_energy_stored() -> i64;

    #[rpc("canExtractEnergy")]
    fn can_extract_energy() -> bool;

    #[rpc("canReceiveEnergy")]
    fn can_receive_energy() -> bool;
}

pub trait ItemHandler: RPCDevice {
    #[rpc("getItemSlotCount")]
    fn get_item_slot_count() -> i64;

    #[rpc("getItemSlotLimit")]
    fn get_item_slot_limit(slot: i64) -> i64;

    #[rpc("getItemStackInSlot")]
    fn get_item_stack_in_slot<T: Deserialize>(slot: i64) -> T;
}

pub trait FluidHandler: RPCDevice {
    #[rpc("getFluidTanks")]
    fn get_fluid_tanks() -> i64;

    #[rpc("getFluidTankCapacity")]
    fn get_fluid_tank_capacity(tank: i64) -> i64;

    #[rpc("getFluidInTank")]
    fn get_fluid_in_tank<T: Deserialize>(tank: i64) -> T;
}

pub trait RedstoneInterface: RPCDevice {
    #[rpc("getRedstoneInput", docs = "block/redstone_interface.md")]
    /// gets the received redstone signal for the specified side.
    fn get_redstone_input(side: &str) -> i64;

    #[rpc("getRedstoneOutput", docs = "block/redstone_interface.md")]
    /// gets the emitted redstone signal for the specified side.
    fn get_redstone_output(side: &str) -> i64;

    #[rpc("setRedstoneOutput", docs = "block/redstone_interface.md")]
    /// sets the emitted redstone signal for the specified side.
    fn set_redstone_output(side: &str, val: i64);
}

/// A device capable of playing sounds.
pub trait SoundInterface: RPCDevice {
    /// returns a list of available sound effects matching the given name. Note that the number of results is limited, so overly generic queries will result in truncated results.
    #[rpc("findSound", docs = "item/sound_card.md")]
    fn find_sound(name: &str) -> Vec<String>;

    #[rpc("playSound", docs = "item/sound_card.md")]
    /// plays back the sound effect with the specified name.
    fn play_sound(name: &str);
}

pub trait FileImportExport: RPCDevice {
    #[rpc("requestImportFile")]
    fn request_import_file() -> bool;

    #[rpc("beginImportFile")]
    fn begin_import_file() -> Option<ImportFileInfo>;

    #[rpc("readImportFile")]
    fn read_import_file() -> Option<Vec<u8>>;

    #[rpc("beginExportFile")]
    fn begin_export_file(name: &str);

    #[rpc("writeExportFile")]
    fn write_export_file(data: &[u8]);

    #[rpc("finishExportFile")]
    fn finish_export_file();

    #[rpc("reset")]
    fn reset();
}

define_device!(
    RedstoneDevice,
    "redstone",
    "A device capable of interacting with redstone",
    [RedstoneInterface]
);
define_device!(
    SoundCard,
    "sound",
    "A device capable of playing sounds",
    [SoundInterface]
);
define_device!(
    FileImportExportCard,
    "file_import_export",
    "A device capable of importing and exporting files",
    [FileImportExport]
);
