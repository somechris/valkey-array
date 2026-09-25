//! Vendor agnostic aliases

pub use valkey_module::RedisModuleTypeMethods as RespModuleTypeMethods;
pub use valkey_module::raw::RedisModuleIO as RespModuleIO;

#[cfg(test)]
pub use redis::FromRedisValue as FromRespValue;
#[cfg(test)]
pub use redis::RedisResult as RespResult;
#[cfg(test)]
pub use redis::ToRedisArgs as ToRespArgs;
