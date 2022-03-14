[![Crates.io](https://img.shields.io/crates/v/miku-rpc?style=flat-square)](https://crates.io/crates/miku-rpc)
[![docs.rs](https://img.shields.io/docsrs/miku-rpc/latest?style=flat-square)](https://docs.rs/miku-rpc/latest)

# miku-rpc
an implementation of the RPC device API for [OpenComputers 2](https://github.com/fnuecke/oc2).

named in honor of minecraft's creator, hatsune miku.

## using as a lua module
you can use this library from lua! you can [download it as a datapack](https://cat-girl.gay/miku-datapack.zip) and, after installing it, use it as a lua module from your OC2 vms.
the exposed API is designed to be as similar as possible to the default lua RPC api:
```lua
local DeviceBus = require("libmiku")
local bus = DeviceBus:new("/dev/hvc0")

local redstone = bus:find("redstone")
print(redstone:getRedstoneInput("left"))
```