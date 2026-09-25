//! The data type that gets registered in Valkey for arrays
//!
//! The Rust implementation for arrays is it [`crate::Array`].

use crate::array::{Array, ArrayType};
use crate::utils::agnostic::{RespModuleIO, RespModuleTypeMethods};
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use valkey_module::error::Error;
use valkey_module::native_types::ValkeyType;
use valkey_module::{logging, raw};

/// The data type for Valkey itself
pub static VKARRAY: ValkeyType = ValkeyType::new(
    "vkarray__",
    0,
    RespModuleTypeMethods {
        version: valkey_module::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(vkarray_rdb_load),
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

/// Saves an array
extern "C" fn vkarray_rdb_save(rdb: *mut RespModuleIO, value: *mut c_void) {
    unsafe {
        let array = &*value.cast::<Array>();
        raw::save_unsigned(rdb, array.count() as u64);
        for (position, value) in array.iter() {
            raw::save_unsigned(rdb, *position);
            raw::save_redis_string(rdb, value);
        }
    }
}

/// Loads an array from encoding version 0
fn vkarray_rdb_load_version_0(rdb: *mut RespModuleIO) -> Result<Array, Error> {
    let mut array = Array::new();

    let count = raw::load_unsigned(rdb)?;
    for _ in 0..count {
        let position = raw::load_unsigned(rdb)?;
        let value = raw::load_string(rdb)?;
        array.set(position, value);
    }

    Ok(array)
}

unsafe extern "C" fn vkarray_rdb_load(rdb: *mut RespModuleIO, encver: c_int) -> *mut c_void {
    if encver != 0 {
        logging::log_warning(format!("Cannot load version {encver}"));
        return null_mut();
    }

    match vkarray_rdb_load_version_0(rdb) {
        Ok(array) => Box::into_raw(Box::new(array)).cast::<c_void>(),
        Err(err) => {
            logging::log_warning(format!("Failed to load array: {err}"));
            null_mut()
        }
    }
}
