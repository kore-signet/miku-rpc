use miku_rpc::{bus::DeviceBus, Call, Response};
use miniserde::json;

fn main() -> std::io::Result<()> {
    let mut bus = DeviceBus::new("/dev/hvc0")?;
    let redstone_side = std::env::args().nth(1).unwrap();
    let redstone_id = bus.find("redstone")?.unwrap();

    let response: Response<json::Value> = bus.call(&Call::invoke(
        &redstone_id,
        "getRedstoneInput",
        vec![&redstone_side],
    ))?;

    println!("{}", json::to_string(&response.data));

    Ok(())
}
