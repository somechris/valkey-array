use std::collections::HashMap;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::{raw, valkey_module, Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};
use valkey_module::native_types::ValkeyType;

static ARRAY_TYPE: ValkeyType = ValkeyType::new(
    "vkarray",
    0,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
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
struct Array {
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

fn arget(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;
    let position = &args.next_u64()?;

    if args.next().is_some() {
        return Err(ValkeyError::WrongArity);
    }

    ctx.log_warning(&format!("Running ARGET for {key_name} @ {position}"));


    let key = ctx.open_key(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&ARRAY_TYPE) else {
        return Err(ValkeyError::WrongType)
    };

    let Some(array) = maybe_array else {
        return Ok(ValkeyValue::Null)
    };

    let value = match array.get(position) {
        Some(ref_value) => ValkeyValue::BulkValkeyString(ref_value.clone()),
        None => ValkeyValue::Null
    };

    Ok(value)
}

fn arset(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    let mut args = args.into_iter().skip(1);
    let key_name = &args.next_arg()?;
    let position = &args.next_u64()?;
    let value = &args.next_arg()?;

    if args.next().is_some() {
        return Err(ValkeyError::WrongArity);
    }

    ctx.log_warning(&format!("Running ARSET for {key_name} @ {position} = {value}"));


    let key = ctx.open_key_writable(key_name);
    let Ok(maybe_array) = key.get_value::<Array>(&ARRAY_TYPE) else {
        return Err(ValkeyError::WrongType)
    };

    let new_slot_count = match maybe_array {
        Some(array) => array.set(position, value),
        None => {
            let mut array = Array::new();
            let ret = array.set(position, value);
            if key.set_value(&ARRAY_TYPE, array).is_err() {
                return Err(ValkeyError::Str("Failed to set value"));
            }
            ret
        },
    };

    Ok(ValkeyValue::Integer(new_slot_count as i64))
}

valkey_module! {
    name: "vkarray",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [],
    commands: [
        ["arget", arget, "", 1, 1, 1],
        ["arset", arset, "", 1, 1, 1],
    ],
}
