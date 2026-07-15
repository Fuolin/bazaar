use std::io::Write;
use std::fs;
use std::path::PathBuf;

use toml;

use crate::{monitors::Monitor, writer::Block};
mod monitors;

mod writer;
mod commander;
//use rustbus
/*
[liuno@liuno ~]$ sudo busctl monitor org.freedesktop.NetworkManager
[liuno@liuno ~]$ sudo busctl monitor net.connman.iwd
[liuno@liuno ~]$ sudo busctl monitor org.bluez
[liuno@liuno ~]$ sudo busctl monitor org.freedesktop.Notifications

notify-send "通知标题" "通知正文内容"
*/

struct Register {
    fdm:Vec<u8>,
    fds:Vec<libc::pollfd>,
    monitors:Vec<Box<dyn Monitor>>,
    commander:commander::Commander,
    writer:writer::Writer,
}
impl Register {
    fn new(writer:writer::Writer) -> Self {
        Self {
            fdm:Vec::new(),
            fds:vec![libc::pollfd { fd: -1, events: libc::POLLIN, revents: 0 }],//基础FD
            monitors:Vec::new(),
            commander:commander::Commander::new(),
            writer,
        }
    }

    fn regist(&mut self,fds: Vec<libc::pollfd>,monitor:Box<dyn Monitor>,block:Block,command:String){
        self.fdm.push(fds.len() as u8);
        self.fds.extend(fds);
        self.monitors.push(monitor);
        self.writer.add_block(block);
        self.commander.add_command(command);
    }

    fn write_from_fd(&mut self,fd:libc::pollfd, out: &mut impl Write){
        for (i, m) in self.fdm.iter().enumerate() {
            for j in 0..*m {
                if self.fds[1 + i + j as usize].fd == fd.fd {
                    self.writer.update_block(out,i,self.monitors[i].get_data());
                    return;
                }
            }
        }
    }

    fn command_from_selctor(&mut self){
        let selector = self.writer.get_selector() as usize;
        self.commander.command(selector);
    }

    fn set_selector(&mut self,out: &mut impl Write,selector:u16){
        self.writer.set_selector(out,selector);
    }
}

unsafe extern "C" {
    fn getpwuid(uid: u32) -> *mut libc::passwd;
    fn getuid() -> u32;
}
fn get_config_path() -> Option<PathBuf> {
    let mut home: PathBuf = unsafe {

        let uid = getuid();

        let pwd = getpwuid(uid);
        if pwd.is_null() {
            return None;
        }
        let home_cstr = (*pwd).pw_dir;
        if home_cstr.is_null() {
            return None;
        }

        let home = match std::ffi::CStr::from_ptr(home_cstr).to_str() {
            Ok(s) => s,
            Err(_) => return None,
        };
        PathBuf::from(home)
    };
    home.push(".config");
    home.push("bazaar");
    home.push("config.toml");
    
    Some(home)
}

fn load_config(out: &mut impl Write) -> Option<Register>{
    let p = match get_config_path() {
        Some(path) => path,
        None => {
            println!("config is none");
            return None
        }
    };

    if !p.is_file() {
        create_default_config(&p);
    }

    let text = match fs::read_to_string(p) {
        Ok(t) => t,
        Err(_) => return None,
    };

    let config: toml::Value = match toml::from_str(&text) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let layout = config.get("layout").expect("缺少 [layout] 配置");

    let comp_list = config.get("components")
    .and_then(|v| v.as_array())
    .expect("[[components]] 配置错误");

    let w = writer::Writer::start(out, layout.to_string());

    let mut r = Register::new(w);
    for comp in comp_list {
        let (monitor,fds):(Box<dyn Monitor>,Vec<libc::pollfd>)= match comp["type"].as_str() {
            Some(s) => {
                match s {
                    "time" => {
                        let (t, fds) = monitors::Timer::new();
                        (Box::new(t),fds)
                    }
                    "brightness" => {
                        let (t, fds) = monitors::BrightnessMonitor::new();
                        (Box::new(t),fds)
                    }
                    "alsa" => {
                        let (t, fds) = monitors::ALSAMonitor::new();
                        (Box::new(t),fds)
                    }
                    "network" => {
                        let (t, fds) = monitors::NmMonitor::new();
                        (Box::new(t),fds)
                    }
                    "bluetooth" => {
                        let (t, fds) = monitors::BtMonitor::new();
                        (Box::new(t),fds)
                    }
                    "workspace" => {
                        let (t, fds) = monitors::WSMonitor::new();
                        (Box::new(t),fds)
                    }
                    _ => continue,
                }
            }
            None => {
                continue;
            }
        };

        let command = match comp["command"].as_str() {
            Some(c) => c.to_string(),
            None => continue
        };

        let (x,dx) = match comp["x"].as_str() {
            Some(x_dx) => {
                (0,0)
            }
            None => {
                (0,0)
            }
        };

        let (y,dy) = match comp["y"].as_str() {
            Some(y_dy) => {
                (0,0)
            }
            None => {
                (0,0)
            }
        };

        let l = match comp["longth"].as_integer() {
            Some(l) => {
                40
            }
            None => {
                40
            }
        };

        let block = Block::new(x, dx, y, dy, l);

        r.regist(fds, monitor, block, command);

        //let w = match
    }
    

    Some(r)
}

fn create_default_config(path: &PathBuf) {

}

/*
[
layout
]
rows = 2

[[
components
]]
type = "time" or "brightness" "alsa" "network" "bluetooth" "workspace"

command = "sh command"

x = a%b
y = c%d
longth = l
#width = w
 */

fn main() {
    let _terminal_guard = writer::TerminalGuard::new().expect("终端初始化失败");
    mainloop();
}

fn mainloop() {
    let mut out = std::io::stdout().lock();

    // 从配置文件创建注册表
    let mut register = match load_config(&mut out) {
        Some(r ) => r,
        None => {
            println!("register is none");
            panic!()
        }
    };

    // 主循环
    loop {
        let ret = unsafe { libc::poll(register.fds.as_mut_ptr(), register.fds.len() as u64, -1) };
        if ret < 0 { 
            if ret == -1  {
                let err = unsafe { *libc::__errno_location() };
                if err == libc::EINTR {
                    continue;
                }
            }
            continue;
        }

        // 按键退出
        if register.fds[0].revents & libc::POLLIN != 0 {
            let mut buf = [0u8; 3];
            let n = unsafe { libc::read(register.fds[0].fd, buf.as_mut_ptr() as _, 3) };
            if n > 0 {
                if buf[0] == b'q' { break; }
                if buf[0] == b'n' { notepad(); }
                if buf[0] == b'j' { register.set_selector(&mut out,1); }
                if buf[0] == b'e' { register.command_from_selctor(); }
            }
            register.fds[0].revents = 0;
        }

        for i in 1..register.fds.len() {
            if register.fds[i].revents & libc::POLLIN != 0 {
                register.write_from_fd(register.fds[i], &mut out);
                register.fds[i].revents = 0;
            }
        }

        register.writer.check_size(&mut out);
        out.flush().unwrap();
    }
}


fn notepad(){

}


//消息通知