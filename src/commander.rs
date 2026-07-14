pub struct Commander {
    commands: Vec<String>,
}

impl Commander {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: String) {
        self.commands.push(command);
    }

    pub fn command(&self,i:usize) {
//线程管理
        if i < self.commands.len() {
            let _command = &self.commands[i];
        }
    }
}