pub struct Commander {
    pub commands: Vec<String>,
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

    pub fn command(&self,i:usize) -> &str {
        if i < self.commands.len() {
            return &self.commands[i]
        }
        return "none";
    }
}
//电源选项，风扇转速管理脚本，快捷键显示。。鼠标功能