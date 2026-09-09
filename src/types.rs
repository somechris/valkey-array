use std::collections::HashMap;
use valkey_module::ValkeyString;
use valkey_module::native_types::ValkeyType;

pub static ARRAY_TYPE: ValkeyType = ValkeyType::new(
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

#[derive(Default, Debug)]
pub struct Array {
    values: HashMap<u64, ValkeyString>
}

impl Array {
    pub fn new() -> Self {
        Array::default()
    }

    pub fn get(&self, position: &u64) -> Option<&ValkeyString> {
        self.values.get(position)
    }

    pub fn set(&mut self, position: &u64, value: &ValkeyString) -> usize {
        if self.values.insert(*position, value.clone()).is_some() {
            // The position already had a value, so it's not a new slot
            0
        } else {
            // The position previously did not have a value, so it's a new slot
            1
        }
    }
}