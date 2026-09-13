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
    acl_categories: [
        "array",
    ]
    commands: [
        ["arcount", commands::arcount, "readonly fast", 1, 1, 1, "read fast array"],
        ["ardel", commands::ardel, "write fast", 1, 1, 1, "write fast array"],
        ["ardelrange", commands::ardelrange, "write", 1, 1, 1, "write array"],
        ["arget", commands::arget, "readonly fast", 1, 1, 1, "read fast array"],
        ["argetrange", commands::argetrange, "readonly", 1, 1, 1, "read array"],
        ["arinfo", commands::arinfo, "readonly fast", 1, 1, 1, "read fast array"],
        ["arinsert", commands::arinsert, "write fast deny-oom", 1, 1, 1, "write fast array"],
        ["arlen", commands::arlen, "readonly fast", 1, 1, 1, "read fast array"],
        ["armget", commands::armget, "readonly", 1, 1, 1, "read array"],
        ["arnext", commands::arnext, "readonly fast", 1, 1, 1, "read fast array"],
        ["arring", commands::arring, "write fast deny-oom", 1, 1, 1, "write fast array"],
        ["arseek", commands::arseek, "write fast", 1, 1, 1, "write fast array"],
        ["arset", commands::arset, "write fast deny-oom", 1, 1, 1, "write fast array"],
    ],
}
