use miku_rpc::wrappers::*;
use miku_rpc::DeviceBus;

fn main() -> std::io::Result<()> {
    let mut bus = DeviceBus::new("/dev/hvc0")?;
    let redstone_side = std::env::args().nth(1).unwrap();
    let redstone_card = bus.wrap::<RedstoneDevice>()?.unwrap();

    println!(
        "{}",
        redstone_card.get_redstone_input(&mut bus, &redstone_side)?
    );
    println!(
        "{:?}",
        redstone_card.set_redstone_output(&mut bus, &redstone_side, 15)?
    );

    Ok(())
}
