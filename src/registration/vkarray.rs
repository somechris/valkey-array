//! The data type that gets registered in Valkey for arrays
//!
//! The Rust implementation for arrays is it [`crate::Array`].

use crate::array::Array;
use std::os::raw::c_void;
use valkey_module::native_types::ValkeyType;
use valkey_module::raw;

/// The data type for Valkey itself
pub static VKARRAY: ValkeyType = ValkeyType::new(
    "vkarray__",
    0,
    valkey_module::RedisModuleTypeMethods {
        version: valkey_module::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: None,
        rdb_save: Some(vkarray_rdb_save),
        aof_rewrite: None,
        free: Some(vkarray_free),
        digest: None,
        mem_usage: None,

        // Aux data
        aux_load: None,
        aux_save: None,
        aux_save2: None,
        aux_save_triggers: 0,

        free_effort: None,
        unlink: None,
        copy: None,
        defrag: None,

        copy2: None,
        free_effort2: None,
        mem_usage2: None,
        unlink2: None,
    },
);

extern "C" fn vkarray_free(value: *mut c_void) {
    if value.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(value.cast::<Array>()));
    }
}

// Not yet saving anything
extern "C" fn vkarray_rdb_save(_rdb: *mut raw::RedisModuleIO, _value: *mut c_void) {}
