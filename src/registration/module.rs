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
        ["arcount", commands::arcount, "", 1, 1, 1],
        ["ardel", commands::ardel, "", 1, 1, 1],
        ["arget", commands::arget, "", 1, 1, 1],
        ["arinsert", commands::arinsert, "", 1, 1, 1],
        ["arlen", commands::arlen, "", 1, 1, 1],
        ["arset", commands::arset, "", 1, 1, 1],
    ],
}
