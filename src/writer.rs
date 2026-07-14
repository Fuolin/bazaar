use std::io::{Write, stdout,Result};

use crossterm::{queue, execute};
use crossterm::terminal::{size, Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{MoveTo, Hide, Show};
use crossterm::style::Stylize;

pub struct TerminalGuard;
impl TerminalGuard {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        execute!(stdout(), Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}
#[derive(Clone)]
struct Block {
    x:u16,
    y:u16,
    l:u16,
    latest: [u8; 32],
    str_len: usize,
}

impl Block {
    fn new() -> Self{
        Self{
            x:0,
            y:0,
            l:0,
            latest:[0; 32],
            str_len:0,
        }
    }

    fn update_location(&mut self,x:u16,y:u16,l:u16){
        self.x = x;
        self.y = y;
        self.l = l;
    }

    fn update(&mut self,text:String){
        self.str_len = text.as_bytes().len().min(31);
        self.latest[0..self.str_len].copy_from_slice(&text.as_bytes()[0..self.str_len]);
    }

    fn get_latest(&self) -> &str {
        std::str::from_utf8(&self.latest[0..self.str_len]).unwrap()
    }
}

pub struct Writer {
    rows:u16,
    cols:u16,
    blocks:Vec<Block>,
    selector:u16,
    layout:String,

}
impl Writer {
    pub fn start(out: &mut impl Write,layout:String) -> Self {
        let (cols,rows) = size().unwrap_or((80,24));
        //从layout获取block的数量
        let mut writer = Self {
            rows:rows,
            cols:cols,
            blocks:vec![Block::new();10],
            selector:0,
            layout:layout,
        };
        writer.update_all(out);

        writer
    }

    fn update_all(&mut self,out: &mut impl Write){
        //重新计算blocks的位置和长度
        //全量绘制
        self.print_background(out);
        //从layout获取block的位置
    }

    fn print_background(&mut self,out: &mut impl Write) {
        //从layout获取background的内容
    }

    pub fn update_block(&mut self,out: &mut impl Write, i:usize, latest:String){
        if  self.blocks[i].get_latest() == latest{
            return;
        }
        self.blocks[i].update(latest);
        print_block(out,&self.blocks[i],i == self.selector as usize);
    }

    pub fn check_size(&mut self,out: &mut impl Write){
        let (cols,rows) = size().unwrap_or((80,24));
        if cols != self.cols || rows != self.rows{
            let _ = queue!(out,Clear(ClearType::All),MoveTo(0,0));
            self.cols = cols;
            self.rows = rows;
            self.update_all(out);
        }
    }

    pub fn set_selector(&mut self,out: &mut impl Write,selector:u16){
        self.selector = selector;
    }

    pub fn get_selector(&self) -> u16{
        self.selector
    }
}

// 局部打印（安全UTF-8截断，零堆分配）
fn print_block(out: &mut impl Write, block: &Block, is_selected: bool) {
    let _ = queue!(out, MoveTo(block.x, block.y));
    let l = block.l as usize;

    if l == 0 {
        return;
    }
    
    // 安全截断UTF-8字符串，不产生堆分配
    let mut chars = block.get_latest().chars();
    let mut byte_len = 0;
    for _ in 0..l {
        match chars.next() {
            Some(c) => byte_len += c.len_utf8(),
            None => break,
        }
    }
    
    let truncated = &block.get_latest()[..byte_len];
    if is_selected {
        let _ = write!(out, "{:l$}", truncated.underlined());
    } else {
        let _ = write!(out, "{:l$}", truncated);
    }
}
/* 
// 绘制背景
fn print_background(out: &mut impl Write, cols: &u16) -> (usize, usize) {
    let content_width = cols.saturating_sub(3) as usize;
    let border = "─".repeat(content_width);
    let right = content_width.saturating_sub(25);
    let right1 = right / 2;
    let right2 = right - right1;

    let total_space = content_width.saturating_sub(45);
    let pad_left = total_space / 2;
    let pad_right = total_space - pad_left;

    let _ = queue!(out, MoveTo(0, 0));
    let _ = writeln!(out, "╭{border}╮\r");
    let _ = writeln!(out, "│ {}     {}     {}     {} {:<right1$} {} {:<right2$} │\r",
        Icons::BRIGHT, Icons::VOL, Icons::MIC, Icons::BT, "", Icons::WIFI, "");
    let _ = writeln!(out, "│ {} workspace:   {:pad_left$}bazaar{:pad_right$} {}                     │\r",
        Icons::WS, "", "", Icons::TIME);
    let _ = writeln!(out, "╰{border}╯\r");

    (right1, right2)
}
*/