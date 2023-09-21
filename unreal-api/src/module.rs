use std::collections::{HashMap, HashSet};

use crate::{
    core::{EntityEvent, SendEntityEvent, UnrealCore},
    ecs::{
        event::Event,
        prelude::{Events, System},
        schedule::{ExecutorKind, IntoSystemConfigs, Schedule, ScheduleLabel, Schedules},
        system::Resource,
        world::FromWorld,
    },
    editor_component::AddSerializedComponent,
    ffi::UnrealBindings,
    plugin::Plugin,
    schedules::{
        EventRegistration, Main, MainScheduleOrder, PostUpdate, PreUpdate, Startup, Update,
    },
};
use bevy_utils::tracing::info;
use unreal_reflect::{registry::ReflectDyn, uuid, TypeUuid, World};

pub static mut MODULE: Option<Global> = None;
pub struct Global {
    pub core: UnrealCore,
    pub module: Box<dyn UserModule>,
}

pub trait InitUserModule {
    fn initialize() -> Self;
}

pub type EmptySystem = &'static dyn System<In = (), Out = ()>;
#[macro_export]
macro_rules! register_components {
    ($($ty: ty,)* => $module: expr) => {
        $(
            $module.register_component::<$ty>();
        )*
    };
}
// TODO: Error on duplicated guids
#[macro_export]
macro_rules! register_editor_components {
    ($($ty: ty,)* => $module: expr) => {
        $(
            $module.register_editor_component::<$ty>();
        )*
    };
}

#[macro_export]
macro_rules! register_events {
    ($($ty: ty,)* => $module: expr) => {
        $(
            $module.register_event::<$ty>();
        )*
    };
}
pub trait RegisterReflection {
    fn register_reflection(registry: &mut ReflectionRegistry);
}

pub trait RegisterSerializedComponent {
    fn register_serialized_component(registry: &mut ReflectionRegistry);
}

pub trait RegisterEvent {
    fn register_event(registry: &mut ReflectionRegistry);
}

#[derive(Default)]
pub struct ReflectionRegistry {
    pub uuid_set: HashSet<uuid::Uuid>,
    pub reflect: HashMap<uuid::Uuid, Box<dyn ReflectDyn>>,
    pub insert_serialized_component: HashMap<uuid::Uuid, Box<dyn AddSerializedComponent>>,
    pub send_entity_event: HashMap<uuid::Uuid, Box<dyn SendEntityEvent>>,
    pub editor_components: HashSet<uuid::Uuid>,
    pub events: HashSet<uuid::Uuid>,
}

impl ReflectionRegistry {
    pub fn register<T>(&mut self)
    where
        T: RegisterReflection + TypeUuid + 'static,
    {
        if self.uuid_set.contains(&T::TYPE_UUID) {
            panic!(
                "Duplicated UUID {} for {}",
                T::TYPE_UUID,
                std::any::type_name::<T>()
            );
        }
        T::register_reflection(self);
        self.uuid_set.insert(T::TYPE_UUID);
    }
}

pub struct Module {
    pub reflection_registry: ReflectionRegistry,
    pub(crate) world: World,
}

impl Module {
    pub fn new() -> Self {
        println!("About to call Module::new()");
        let mut world = World::new();

        let mut startup = Schedule::new();
        startup.set_executor_kind(ExecutorKind::SingleThreaded);
        world.add_schedule(startup, Startup);

        let mut main_schedule = Schedule::new();
        main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        world.add_schedule(main_schedule, Main);
        println!("About to call init_resource on MainScheduleOrder");
        world.init_resource::<MainScheduleOrder>();

        world.add_schedule(Schedule::new(), EventRegistration);
        world.add_schedule(Schedule::new(), PreUpdate);
        world.add_schedule(Schedule::new(), Update);
        world.add_schedule(Schedule::new(), PostUpdate);

        Self {
            reflection_registry: ReflectionRegistry::default(),
            world,
        }
    }
    pub fn insert_resource(&mut self, resource: impl Resource) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        info!("System {schedule:?} added.");
        let mut schedules = self.world.resource_mut::<Schedules>();

        if let Some(schedule) = schedules.get_mut(&schedule) {
            schedule.add_systems(systems);
        } else {
            let mut new_schedule = Schedule::new();
            new_schedule.add_systems(systems);
            schedules.insert(schedule, new_schedule);
        }

        self
    }

    pub fn add_schedule(&mut self, schedule: Schedule, label: impl ScheduleLabel) -> &mut Self {
        self.world.add_schedule(schedule, label);
        self
    }

    pub fn register_component<T>(&mut self)
    where
        T: RegisterReflection + TypeUuid + 'static,
    {
        T::register_reflection(&mut self.reflection_registry);
        self.reflection_registry.uuid_set.insert(T::TYPE_UUID);
        info!("Component was registered: {:?}", T::TYPE_UUID);
    }

    pub fn register_editor_component<T>(&mut self)
    where
        T: RegisterReflection + RegisterSerializedComponent + TypeUuid + 'static,
    {
        T::register_reflection(&mut self.reflection_registry);
        T::register_serialized_component(&mut self.reflection_registry);
        self.reflection_registry.uuid_set.insert(T::TYPE_UUID);
        self.reflection_registry
            .editor_components
            .insert(T::TYPE_UUID);
        info!("Editor component was registered: {:?}", T::TYPE_UUID);
    }

    pub fn register_event<T>(&mut self)
    where
        T: RegisterReflection + RegisterEvent + Event + TypeUuid + Send + Sync + 'static,
    {
        self.reflection_registry.uuid_set.insert(T::TYPE_UUID);
        T::register_event(&mut self.reflection_registry);
        T::register_reflection(&mut self.reflection_registry);

        self.add_event::<EntityEvent<T>>();
        self.add_event::<T>();
        info!("Event was registered: {:?}", T::TYPE_UUID);
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_event<T: Event>(&mut self) -> &mut Self {
        info!("Adding an event");
        if !self.world.contains_resource::<Events<T>>() {
            self.init_resource::<Events<T>>()
                .add_systems(EventRegistration, Events::<T>::update_system);
        }
        self
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

pub trait UserModule {
    fn initialize(&self, module: &mut Module);
}
pub static mut BINDINGS: Option<UnrealBindings> = None;

#[macro_export]
macro_rules! implement_unreal_module {
    ($module: ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn register_unreal_bindings(
            bindings: $crate::ffi::UnrealBindings,
            rust_bindings: *mut $crate::ffi::RustBindings,
        ) -> u32 {
            std::panic::set_hook(Box::new(|panic_info| {
                let bt = std::backtrace::Backtrace::force_capture();
                log::error!("{}", bt);
                let info = panic_info
                    .payload()
                    .downcast_ref::<&'static str>()
                    .copied()
                    .or(panic_info
                        .payload()
                        .downcast_ref::<String>()
                        .map(String::as_str));

                if let Some(s) = info {
                    let location = panic_info.location().map_or("".to_string(), |loc| {
                        format!("{}, at line {}", loc.file(), loc.line())
                    });
                    log::error!("Panic: {} => {}", location, s);
                } else {
                    log::error!("panic occurred");
                }
            }));
            println!("About to initialize BINDINGS with UnrealBindings");
            $crate::module::BINDINGS = Some(bindings);
            let _ = $crate::log::init();

            let r = std::panic::catch_unwind(|| unsafe {
                println!("About to box the result of initializing InitUserModule::initialize()");
                let module = Box::new(<$module as $crate::module::InitUserModule>::initialize());
                println!("About to call UnrealCore::new");
                let core = $crate::core::UnrealCore::new(module.as_ref());

                println!("About to initialize MODULe with the boxed module and UnrealCore");
                $crate::module::MODULE = Some($crate::module::Global { core, module });
                println!("About to return RustBindings");
                $crate::ffi::RustBindings {
                    retrieve_uuids: $crate::core::retrieve_uuids,
                    tick: $crate::core::tick,
                    begin_play: $crate::core::begin_play,
                    unreal_event: $crate::core::unreal_event,
                    reflection_fns: $crate::core::create_reflection_fns(),
                    allocate_fns: $crate::core::create_allocate_fns(),
                    send_actor_event: $crate::core::send_actor_event,
                }
            });
            match r {
                Ok(bindings) => {
                    *rust_bindings = bindings;
                    1
                }
                Err(_) => 0,
            }
        }
    };
}

pub fn bindings() -> &'static UnrealBindings {
    unsafe { BINDINGS.as_ref().unwrap() }
}
