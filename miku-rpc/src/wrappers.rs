use crate::types::{ImportFileInfo, MoveDirection, RobotActionResult, RotationDirection};
use miku_macros::{define_device, rpc};
use miniserde::Deserialize;
use std::io;
use std::thread;
use std::time::Duration;

const ROBOT_ACTION_SLEEP: Duration = Duration::from_millis(100);

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

/// An interface that allows for interacting with an energy storage device.
pub trait EnergyStorage: RPCDevice {
    #[rpc("getEnergyStored")]
    fn get_energy_stored() -> i32;

    #[rpc("getMaxEnergyStored")]
    fn get_max_energy_stored() -> i32;

    #[rpc("canExtractEnergy")]
    fn can_extract_energy() -> bool;

    #[rpc("canReceiveEnergy")]
    fn can_receive_energy() -> bool;
}

/// An interface that allows for interacting with items.
pub trait ItemHandler: RPCDevice {
    #[rpc("getItemSlotCount")]
    fn get_item_slot_count() -> i32;

    #[rpc("getItemSlotLimit")]
    fn get_item_slot_limit(slot: i32) -> i32;

    #[rpc("getItemStackInSlot")]
    fn get_item_stack_in_slot<T: Deserialize>(slot: i32) -> T;
}

/// An interface that allows for interacting with fluid tanks.
pub trait FluidHandler: RPCDevice {
    #[rpc("getFluidTanks")]
    fn get_fluid_tanks() -> i32;

    #[rpc("getFluidTankCapacity")]
    fn get_fluid_tank_capacity(tank: i32) -> i32;

    #[rpc("getFluidInTank")]
    fn get_fluid_in_tank<T: Deserialize>(tank: i32) -> T;
}

/// An interface that allows for interacting with redstone signals.
pub trait RedstoneInterface: RPCDevice {
    #[rpc("getRedstoneInput", docs = "block/redstone_interface.md")]
    /// gets the received redstone signal for the specified side.
    fn get_redstone_input(side: &str) -> i32;

    #[rpc("getRedstoneOutput", docs = "block/redstone_interface.md")]
    /// gets the emitted redstone signal for the specified side.
    fn get_redstone_output(side: &str) -> i32;

    #[rpc("setRedstoneOutput", docs = "block/redstone_interface.md")]
    /// sets the emitted redstone signal for the specified side.
    fn set_redstone_output(side: &str, val: i32);
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

/// An interface that allows exporting and importing files.
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

/// An interface that allows for the manipulation of blocks in the world.
///
/// The side parameter in the following methods represents a direction from the perspective of the robot. Valid values are: "front", "up" and "down"
pub trait BlockOperationsInterface: RPCDevice {
    #[rpc("excavate", docs = "item/block_operations_module.md")]
    /// tries to break a block in the specified direction. Collected blocks will be inserted starting at the currently selected inventory slot. If the selected slot is full, the next slot will be used, and so on. If the inventory has no space for the dropped block, it will drop into the world.
    /// Returns whether the operation was successful.
    fn excavate(side: &str) -> bool;

    #[rpc("place", docs = "item/block_operations_module.md")]
    /// tries to place a block in the specified direction. Blocks will be placed from the currently selected inventory slot. If the slot is empty, no block will be placed.
    /// Returns whether the operation was successful.
    fn place(side: &str) -> bool;

    #[rpc("durability", docs = "item/block_operations_module.md")]
    /// returns the remaining durability of the module's excavation tool. Once the durability has reached zero, no further excavation operations can be performed until it is repaired.
    fn durability() -> i32;

    #[rpc("repair", docs = "item/block_operations_module.md")]
    /// attempts to repair the module's excavation tool using materials in the currently selected inventory slot. This method will consume one item at a time. Any regular tool may act as the source for repair materials, such as pickaxes and shovels. The quality of the tool directly effects the amount of durability restored.
    /// Returns whether some material could be used to repair the module's excavation tool.
    fn repair() -> bool;
}

/// An interface that allows for the manipulation of inventories in the world.
///
/// The side parameter in the following methods represents a direction from the perspective of the robot. Valid values are: "front", "up" and "down"
pub trait InventoryOperationsInterface: RPCDevice {
    #[rpc("move", docs = "item/inventory_operations.md")]
    /// tries to move the specified number of items from one robot inventory slot to another.
    fn move_stack(from: i32, into: i32, count: i32);

    #[rpc("drop", docs = "item/inventory_operations.md")]
    /// tries to drop items from the specified slot in the specified direction. It will drop items either into an inventory, or the world if no inventory is present.
    /// Returns the number of items dropped.
    fn drop(count: i32, side: &str) -> i32;

    #[rpc("dropInto", docs = "item/inventory_operations.md")]
    /// tries to drop items from the specified slot into the specified slot of an inventory in the specified direction. It will only drop items into an inventory.
    /// Returns the number of items dropped.
    fn drop_into(into: i32, count: i32, side: &str) -> i32;

    #[rpc("take", docs = "item/inventory_operations.md")]
    /// tries to take the specified number of items from the specified direction. It will take items from either an inventory, or the world if no inventory is present.
    /// Returns the number of items taken.
    fn take(count: i32, side: &str) -> i32;

    #[rpc("take_from", docs = "item/inventory_operations.md")]
    /// tries to take the specified number of items from the specified slot from an inventory in the specified direction. It will only take items from an inventory.
    /// Returns the number of items taken.
    fn take_from(from: i32, count: i32, side: &str) -> i32;
}

/// A robit!
pub trait RobotInterface: RPCDevice {
    #[rpc("getEnergyStored", docs = "item/robot.md")]
    /// returns the current amount of energy stored in the robot's internal energy storage.
    fn get_energy_stored() -> i32;

    #[rpc("getMaxEnergyStored", docs = "item/robot.md")]
    /// returns the maximum amount of energy that can be stored in the robot's internal energy storage.
    fn get_max_energy_stored() -> i32;

    #[rpc("getSelectedSlot", docs = "item/robot.md")]
    /// returns the currently selected robot inventory slot. This is used by many modules as an implicit input.
    fn get_selected_slot() -> i32;

    #[rpc("setSelectedSlot", docs = "item/robot.md")]
    /// sets the currently selected robot inventory slot. This is used by many modules as an implicit input.
    fn set_selected_slot(slot: i32);

    #[rpc("getStackInSlot", docs = "item/robot.md")]
    /// gets a description of the item in the specified slot.
    fn get_stack_in_slot<T: Deserialize>(slot: i32) -> T;

    #[rpc("getLastActionId", docs = "item/robot.md")]
    /// returns the opaque id of the last enqueued action. Call this after a successful move_async() or turn_async() call to obtain the id associated with the enqueued action.
    fn get_last_action_id() -> i32;

    #[rpc("getQueuedActionCount", docs = "item/robot.md")]
    /// returns the number of actions currently waiting in the action queue to be processed. Use this to wait for actions to finish when enqueueing fails.
    fn get_queued_action_count() -> i32;

    #[rpc("getActionResult", docs = "item/robot.md")]
    /// returns the result of the action with the specified id. Action ids can be obtained from get_last_action_id(). Only a limited number of past action results are available.
    fn get_action_result(id: i32) -> RobotActionResult;

    #[rpc("move", docs = "item/robot.md")]
    /// tries to enqueue a movement action in the specified direction.
    /// Returns whether the action was enqueued successfully.
    fn move_async(direction: MoveDirection) -> bool;

    #[rpc("turn", docs = "item/robot.md")]
    /// tries to enqueue a turn action in the specified direction.
    /// Returns whether the action was enqueued successfully.
    fn turn_async(direction: RotationDirection) -> bool;

    /// Same as move_async(), but waits until action is succesfully enqueued and completed.
    fn move_wait(&self, bus: &mut crate::DeviceBus, direction: MoveDirection) -> io::Result<bool> {
        while !self.move_async(bus, direction)? {
            thread::sleep(ROBOT_ACTION_SLEEP)
        }
        let id = self.get_last_action_id(bus)?;
        self.wait_for_action(bus, id)
    }

    /// Same as turn_async(), but waits until action is succesfully enqueued and completed.
    fn turn_wait(
        &self,
        bus: &mut crate::DeviceBus,
        direction: RotationDirection,
    ) -> io::Result<bool> {
        while !self.turn_async(bus, direction)? {
            thread::sleep(ROBOT_ACTION_SLEEP)
        }
        let id = self.get_last_action_id(bus)?;
        self.wait_for_action(bus, id)
    }

    /// Waits for an action to complete; returns if it was sucessful or not.
    fn wait_for_action(&self, bus: &mut crate::DeviceBus, action: i32) -> io::Result<bool> {
        let result = loop {
            let result = self.get_action_result(bus, action)?;
            match result {
                RobotActionResult::Success | RobotActionResult::Failure => break result,
                RobotActionResult::Incomplete => thread::sleep(ROBOT_ACTION_SLEEP),
            }
        };

        Ok(result == RobotActionResult::Success)
    }
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
define_device!(Robot, "robot", "A robit!", [RobotInterface]);
define_device!(
    BlockOperationsModule,
    "block_operations",
    "A device capable of manipulating blocks in the world",
    [BlockOperationsInterface]
);
define_device!(
    InventoryOperationsModule,
    "inventory_operations",
    "A device capable of manipulating inventories in the world",
    [InventoryOperationsInterface]
);
