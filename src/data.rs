use std::ffi::c_void;
use std::sync::{Arc, RwLock};
use crate::memory::{Vec3, self};
use crate::offset;
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub head: Vec3,
    pub feet: Vec3,
    pub health: Option<i32>,
    pub alive: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub view_matrix: [f32; 16],
    pub valid: bool,
    pub players: Vec::<Player>,
}
fn read_into(module_base: usize, snapshot:& mut Snapshot) {
    snapshot.valid = false;
    snapshot.players.clear();
    let Some(view_matrix) = memory::read::<[f32; 16]>(module_base + offset::VIEW_MATRIX) else {
        return;
    };
    let array_base = memory::read(module_base + offset::ENTITY_LIST).unwrap_or(0) as usize;
    if array_base == 0 {
        return;
    }
    for i in 0..=offset::MAX_PLAYERS {
        let entity = memory::read(array_base + i * 4).unwrap_or(0) as usize;
        if entity == 0 {
            continue;
        }
        let head = memory::read::<Vec3>(entity + offset::HEAD_POSITION);
        let feet = memory::read::<Vec3>(entity + offset::FEET_POSITION);
        let health = memory::read::<i32>(entity + offset::HEALTH).filter(|h| (-10..=100).contains(h));
        let alive = health.map_or(false, |h| h> 0);
        let (head, feet) = match (head, feet) {
            (Some(h), Some(f)) if pose_is_sane(h, f) => (h, f),
            _ => continue,
        };
        snapshot.players.push(Player {
            head,
            feet,
            health,
            alive,
        });
        snapshot.view_matrix = view_matrix;
        snapshot.valid = true;
    }
}

fn pose_is_sane(head: Vec3, feet: Vec3) -> bool {
    plausible(head) && plausible(feet)
}

fn plausible(v: Vec3) -> bool {
    v.x.is_finite()
        && v.y.is_finite()
        && v.z.is_finite()
}

pub fn spawn(module_base: usize, shared: Arc<RwLock<Snapshot>>) {
    std::thread::spawn(move || {
        loop {
            {
                let mut snapshot = match shared.write() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                read_into(module_base, &mut snapshot);
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    });
}