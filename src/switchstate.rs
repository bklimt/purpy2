use std::collections::HashSet;

pub struct SwitchState {
    on: HashSet<String>,
}

impl SwitchState {
    pub fn new() -> Self {
        SwitchState { on: HashSet::new() }
    }

    fn turn_on(&mut self, s: &str) {
        println!("turning on {s}");
        self.on.insert(s.to_owned());
    }

    fn turn_off(&mut self, s: &str) {
        println!("turning off {s}");
        if self.on.contains(s) {
            self.on.remove(s);
        }
    }

    fn toggle(&mut self, s: &str) {
        println!("toggling {s}");
        if self.on.contains(s) {
            self.on.remove(s);
        } else {
            self.on.insert(s.to_owned());
        }
    }

    fn is_on(&self, s: &str) -> bool {
        return self.on.contains(s);
    }

    fn apply_command(&mut self, s: &str) {
        if s.starts_with("~") {
            self.toggle(&s[1..]);
        } else if s.starts_with("!") {
            self.turn_off(&s[1..]);
        } else {
            self.turn_on(s);
        }
    }

    fn is_condition_true(&self, s: &str) -> bool {
        if s.starts_with("!") {
            return !self.is_on(&s[1..]);
        } else {
            return self.is_on(s);
        }
    }
}
