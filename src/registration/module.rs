//! Registration of the module itself
#![allow(
    missing_docs,
    reason = "The generated code is missing docs, but it is outside of our control"
)]
use crate::commands;
use crate::registration::vkarray::VKARRAY;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::valkey_module;

valkey_module! {
    name: "vkarray",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [VKARRAY],
    commands: [
        ["arcount", commands::arcount, "readonly fast", 1, 1, 1],
        ["ardel", commands::ardel, "write fast", 1, 1, 1],
        ["arget", commands::arget, "readonly fast", 1, 1, 1],
        ["arinfo", commands::arinfo, "readonly fast", 1, 1, 1],
        ["arinsert", commands::arinsert, "write fast deny-oom", 1, 1, 1],
        ["arlen", commands::arlen, "readonly fast", 1, 1, 1],
        ["arnext", commands::arnext, "readonly fast", 1, 1, 1],
        ["arring", commands::arring, "write fast deny-oom", 1, 1, 1],
        ["arseek", commands::arseek, "write fast", 1, 1, 1],
        ["arset", commands::arset, "write fast deny-oom", 1, 1, 1],
    ],
}
