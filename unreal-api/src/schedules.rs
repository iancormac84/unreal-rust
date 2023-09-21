use crate::ecs::{
    schedule::ScheduleLabel,
    system::{Local, Resource},
    world::{Mut, World},
};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Main;

/// The schedule that runs once when the app starts.
/// This is run by the [`Main`] schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

/// The schedule that runs after [`Startup`].
/// This is run by the [`Main`] schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventRegistration;

/// The schedule that contains logic that must run before [`Update`]. For example, a system that reads raw keyboard
/// input OS events into an `Events` resource. This enables systems in [`Update`] to consume the events from the `Events`
/// resource without actually knowing about (or taking a direct scheduler dependency on) the "os-level keyboard event sytsem".
///
/// [`PreUpdate`] exists to do "engine/plugin preparation work" that ensures the APIs consumed in [`Update`] are "ready".
/// [`PreUpdate`] abstracts out "pre work implementation details".
///
/// This is run by the [`Main`] schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreUpdate;

/// The schedule that contains app logic.
/// This is run by the [`Main`] schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// The schedule that contains logic that must run after [`Update`]. For example, synchronizing "local transforms" in a hierarchy
/// to "global" absolute transforms. This enables the [`PostUpdate`] transform-sync system to react to "local transform" changes in
/// [`Update`] without the [`Update`] systems needing to know about (or add scheduler dependencies for) the "global transform sync system".
///
/// [`PostUpdate`] exists to do "engine/plugin response work" to things that happened in [`Update`].
/// [`PostUpdate`] abstracts out "implementation details" from users defining systems in [`Update`].
///
/// This is run by the [`Main`] schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostUpdate;

/// Defines the schedules to be run for the [`Main`] schedule, including
/// their order.
#[derive(Resource, Debug)]
pub struct MainScheduleOrder {
    /// The labels to run for the [`Main`] schedule (in the order they will be run).
    pub labels: Vec<Box<dyn ScheduleLabel>>,
}

impl Default for MainScheduleOrder {
    fn default() -> Self {
        println!("Calling MainScheduleOrder::default()");
        Self {
            labels: vec![
                Box::new(EventRegistration),
                Box::new(PreUpdate),
                Box::new(Update),
                Box::new(PostUpdate),
            ],
        }
    }
}

impl MainScheduleOrder {
    /// Adds the given `schedule` after the `after` schedule
    pub fn insert_after(&mut self, after: impl ScheduleLabel, schedule: impl ScheduleLabel) {
        let index = self
            .labels
            .iter()
            .position(|current| (**current).eq(&after))
            .unwrap_or_else(|| panic!("Expected {after:?} to exist"));
        self.labels.insert(index + 1, Box::new(schedule));
    }
}

impl Main {
    /// A system that runs the "main schedule"
    pub fn run_main(world: &mut World, mut run_at_least_once: Local<bool>) {
        println!("Inside run_main");
        if !*run_at_least_once {
            let _ = world.try_run_schedule(Startup);
            *run_at_least_once = true;
        }

        world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
            for label in &order.labels {
                println!("Running thru schedules inside of Main: {label:?}");
                let _ = world.try_run_schedule(&**label);
            }
        });
    }
}