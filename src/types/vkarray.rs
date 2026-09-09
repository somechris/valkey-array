//! The data type that gets registered in Valkey for arrays
//!
//! The Rust implementation for arrays is it [`crate::types::Array`].

use valkey_module::native_types::ValkeyType;

/// The data type for Valkey itself
pub static VKARRAY: ValkeyType = ValkeyType::new(
    "vkarray",
    0,
    valkey_module::RedisModuleTypeMethods {
        version: valkey_module::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: None,
        rdb_save: None,
        aof_rewrite: None,
        free: None,
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
