//! Implementations of array commands

pub mod utils;

/// Import a command's module and publishes the command
macro_rules! add_command {
    ($name:ident) => {
        pub mod $name;
        pub use $name::$name;
    };
}

add_command!(arcount);
add_command!(ardel);
add_command!(ardelrange);
add_command!(arget);
add_command!(argetrange);
add_command!(arinsert);
add_command!(arinfo);
add_command!(arlen);
add_command!(armget);
add_command!(armset);
add_command!(arnext);
add_command!(arring);
add_command!(arscan);
add_command!(arseek);
add_command!(arset);
