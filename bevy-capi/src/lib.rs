// We have a lot of c-types in here, stop warning about their names!
#![allow(non_camel_case_types)]
// fmt::Debug isn't helpful on FFI types
#![allow(missing_debug_implementations)]
// unreachable_pub warns `#[no_mangle] pub extern fn` in private mod.
#![allow(unreachable_pub)]

use bevy_ecs::prelude::*;
use std::ptr;

#[macro_use]
mod macros;

pub struct bevy_world(World);

ffi_fn! {
    fn bevy_world_new() -> *mut bevy_world {
        Box::into_raw(Box::new(bevy_world(World::new())))
    } ?= ptr::null_mut()
}

pub struct bevy_schedule(Schedule);

ffi_fn! {
    fn bevy_schedule_new() -> *mut bevy_schedule {
        Box::into_raw(Box::new(bevy_schedule(Schedule::new())))
    } ?= ptr::null_mut()
}
