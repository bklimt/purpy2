use std::collections::HashSet;

use log::info;

pub struct SwitchState {
    on: HashSet<String>,
}

impl SwitchState {
    pub fn new() -> Self {
        SwitchState { on: HashSet::new() }
    }

    fn turn_on(&mut self, s: &str) {
        info!("turning on {s}");
        self.on.insert(s.to_owned());
    }

    fn turn_off(&mut self, s: &str) {
        info!("turning off {s}");
        if self.on.contains(s) {
            self.on.remove(s);
        }
    }

    pub fn toggle(&mut self, s: &str) {
        info!("toggling {s}");
        if self.on.contains(s) {
            self.on.remove(s);
        } else {
            self.on.insert(s.to_owned());
        }
    }

    fn is_on(&self, s: &str) -> bool {
        self.on.contains(s)
    }

    pub fn apply_command(&mut self, s: &str) {
        if let Some(toggled) = s.strip_prefix('~') {
            self.toggle(toggled);
        } else if let Some(negated) = s.strip_prefix('!') {
            self.turn_off(negated);
        } else {
            self.turn_on(s);
        }
    }

    pub fn is_condition_true(&self, s: &str) -> bool {
        match s.strip_prefix('!') {
            Some(negated) => !self.is_on(negated),
            None => self.is_on(s),
        }
    }
}
