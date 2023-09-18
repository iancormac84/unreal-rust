#![allow(clippy::missing_safety_doc)]
extern crate self as unreal_api;

pub mod api;
pub use unreal_ffi as ffi;
pub mod core;
pub mod editor_component;
pub mod input;
pub mod log;
mod main_schedule;
pub mod module;
pub mod physics;
pub mod plugin;
pub mod sound;
pub use unreal_api_derive::{Component, Event};

// TODO: Here for the unreal_api_derive macro. Lets restructure this
pub use bevy_ecs as ecs;
pub use glam as math;
pub use main_schedule::*;
pub use unreal_reflect::*;

pub use serde;
pub use uuid;

pub use serde::{Deserialize, Serialize};
pub use serde_json;
