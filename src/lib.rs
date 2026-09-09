pub mod commands;
pub mod types;

use valkey_module::alloc::ValkeyAlloc;
use valkey_module::valkey_module;

valkey_module! {
    name: "vkarray",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [],
    commands: [
        ["arget", commands::arget, "", 1, 1, 1],
        ["arset", commands::arset, "", 1, 1, 1],
    ],
}
