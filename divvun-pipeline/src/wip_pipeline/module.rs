use std::fmt;

use hashbrown::HashMap;

#[derive(Debug)]
pub struct ModuleRegistry {
    pub registry: HashMap<String, Module>,
}

impl fmt::Display for ModuleRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")?;
        write!(f, "Registry: ")?;

        for (name, module) in &self.registry {
            write!(f, "{}, {:?})", name, module)?;
            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub output: String,
}
