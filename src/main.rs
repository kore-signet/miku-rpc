use miku_rpc::{bus::DeviceBus, types::DeviceList, RPCMessage};

fn main() {
    let mut bus = DeviceBus::new("/dev/hvc0").unwrap();
    let device_list: DeviceList = bus.call(&RPCMessage::list()).unwrap();
    println!("{:?}", device_list.data);
}
