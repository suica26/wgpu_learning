use std::sync::{Mutex, MutexGuard, OnceLock};

static INSTANCE: OnceLock<Mutex<ApplicationTime>> = OnceLock::new();

pub struct ApplicationTime {
    pub startup_time: std::time::Instant,
    pub delta_time: f32,
    pub last_frame_time: std::time::Instant,
    pub current_frame_rate: u32,
}

impl ApplicationTime {
    pub fn init() -> Result<(), Mutex<ApplicationTime>> {
        let now = std::time::Instant::now();
        INSTANCE.set(Mutex::new(ApplicationTime {
            startup_time: now,
            delta_time: 0.0,
            last_frame_time: now,
            current_frame_rate: 0,
        }))
    }

    pub fn get_instance() -> MutexGuard<'static, ApplicationTime> {
        INSTANCE.get().unwrap().lock().unwrap()
    }

    pub fn update() {
        let mut instance = ApplicationTime::get_instance();
        let now = std::time::Instant::now();

        instance.delta_time = now.duration_since(instance.last_frame_time).as_secs_f32();
        instance.last_frame_time = now;
        instance.current_frame_rate = (1.0 / instance.delta_time) as u32;
    }
}
