use std::io::Write;

use crate::monitors::Monitor;
mod monitors;

mod init;
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
            fdm:vec![1u8],//基础FD
            fds:vec![libc::pollfd { fd: -1, events: libc::POLLIN, revents: 0 }],//基础FD
            monitors:Vec::new(),
            commander:commander::Commander::new(),
            writer,
        }
    }

    fn regist(&mut self,fds: Vec<libc::pollfd>,monitor:Box<dyn Monitor>,command:String){
        self.fdm.push(fds.len() as u8);
        self.fds.extend(fds);
        self.monitors.push(monitor);
        self.commander.add_command(command);
    }

    fn write_from_fd(&mut self,fd:libc::pollfd, out: &mut impl Write){
        for (i, m) in self.fdm.iter().enumerate().skip(1) {
            for j in 0..*m {
                if self.fds[i + j as usize].fd == fd.fd {
                    self.writer.update_block(out,i - 1,self.monitors[i-1].get_data());
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
fn main() {
    let _terminal_guard = writer::TerminalGuard::new().expect("终端初始化失败");
    mainloop();
}

fn mainloop() {
    let mut out = std::io::stdout().lock();

    let (timer,timer_fd) = monitors::Timer::new();
    let (bright,bright_fd) = monitors::BrightnessMonitor::new();
    let (alsa,alsa_fd) = monitors::ALSAMonitor::new();
    let (nm,nm_fd) = monitors::NmMonitor::new();
    let (bt,bt_fd) = monitors::BtMonitor::new();
    let (ws,ws_fd) = monitors::WSMonitor::new();

    // 初始化writer
    let writer = writer::Writer::start(&mut out,"layout".to_string());
    // 注册
    let mut register = Register::new(writer);

    register.regist(timer_fd, Box::new(timer), "timer".to_string());
    register.regist(bright_fd, Box::new(bright), "brightness".to_string());
    register.regist(alsa_fd, Box::new(alsa), "alsa".to_string());
    register.regist(nm_fd, Box::new(nm), "nm".to_string());
    register.regist(bt_fd, Box::new(bt), "bt".to_string());
    register.regist(ws_fd, Box::new(ws), "ws".to_string());

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