//! Unit def ids and the per-def custom params the rules use, resolved once.

use alloc::vec;
use alloc::vec::Vec;
use core::cell::OnceCell;

use partsneeded_shared::BodyPart;

/// The unit defs the rules and unit scripts tell apart.
#[derive(Clone, Copy, Debug)]
pub struct Defs {
    pub wisp: i32,
    pub npcwisp: i32,
    pub gate: i32,
    pub lever: i32,
    pub plate: i32,
    pub cage: i32,
    pub light: i32,
    /// Indexed by `BodyPart as usize`.
    pub parts: [i32; 6],
}

impl Defs {
    fn resolve() -> Self {
        let id = |name: &str| spring::get_unit_def_id_by_name(name).unwrap_or(-1);
        Self {
            wisp: id("wisp"),
            npcwisp: id("npcwisp"),
            gate: id("gatesmoth"),
            lever: id("lever"),
            plate: id("plate"),
            cage: id("cage"),
            light: id("light"),
            parts: BodyPart::ALL.map(|part| id(part.name())),
        }
    }

    /// The def of a level unit, by its name in game_custom_spawn.lua.
    pub fn by_name(&self, name: &str) -> Option<i32> {
        let def = match name {
            "wisp" => self.wisp,
            "npcwisp" => self.npcwisp,
            "gatesmoth" => self.gate,
            "lever" => self.lever,
            "plate" => self.plate,
            "cage" => self.cage,
            "light" => self.light,
            _ => BodyPart::ALL
                .into_iter()
                .find(|part| part.name() == name)
                .map(|part| self.part_def(part))?,
        };
        (def >= 0).then_some(def)
    }

    pub fn body_part(&self, def: i32) -> Option<BodyPart> {
        BodyPart::ALL
            .into_iter()
            .find(|part| self.parts[*part as usize] == def)
    }

    pub fn part_def(&self, part: BodyPart) -> i32 {
        self.parts[part as usize]
    }
}

/// How the authored gadgets treat a def's collision, from its custom params.
#[derive(Clone, Copy, Debug, Default)]
pub struct Collision {
    /// `wall` is set and not "false"/"0": pinned to the floor height.
    pub pinned_wall: bool,
    /// `wall` without `wall_blocking` (unit_collisions.lua).
    pub scenery_wall: bool,
    /// `trigger` (unit_plate.lua).
    pub trigger: bool,
}

impl Collision {
    fn resolve(def: i32) -> Self {
        // The engine reports a missing key as an empty string, which Lua sees
        // as nil.
        let param = |key: &str| {
            spring::unit_defs::get_unit_def_custom_param(def, key)
                .ok()
                .filter(|value| !value.is_empty())
        };
        let wall = param("wall");
        Self {
            pinned_wall: wall
                .as_deref()
                .is_some_and(|value| value != "false" && value != "0"),
            scenery_wall: wall.is_some() && param("wall_blocking").is_none(),
            trigger: param("trigger").is_some(),
        }
    }
}

/// Every unit def's collision rules, indexed by def id.  Def ids run from 1
/// up to the last def the engine can name.
fn resolve_collisions() -> Vec<Collision> {
    let mut collisions = vec![Collision::default()];
    while spring::unit_defs::get_unit_def_name_bytes(collisions.len() as i32).is_ok() {
        collisions.push(Collision::resolve(collisions.len() as i32));
    }
    collisions
}

thread_local! {
    static DEFS: OnceCell<Defs> = const { OnceCell::new() };
    static COLLISIONS: OnceCell<Vec<Collision>> = const { OnceCell::new() };
}

pub fn defs() -> Defs {
    DEFS.with(|defs| *defs.get_or_init(Defs::resolve))
}

pub fn collision(def: i32) -> Collision {
    COLLISIONS.with(|collisions| {
        let collisions = collisions.get_or_init(resolve_collisions);
        usize::try_from(def)
            .ok()
            .and_then(|def| collisions.get(def).copied())
            .unwrap_or_default()
    })
}

/// The def id of a unit, or -1.
pub fn unit_def(unit: i32) -> i32 {
    spring::get_unit_def_id(unit).map_or(-1, |def| def.0)
}
