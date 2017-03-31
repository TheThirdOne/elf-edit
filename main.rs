extern crate pancurses;

use std::fs::File;
use std::io::Read;
use pancurses::*;
use std::env;
use std::cmp;

fn main() {
  let window = initscr();
  start_color();
  use_default_colors();
  
  noecho();

  window.keypad(true);
  init_pair(1, COLOR_BLACK, COLOR_YELLOW);
  init_pair(2, COLOR_WHITE, COLOR_GREEN);
  init_pair(3, COLOR_WHITE, COLOR_CYAN);
  init_pair(4, COLOR_WHITE, COLOR_BLUE);
  init_pair(5, COLOR_WHITE, COLOR_RED);
  init_pair(6, COLOR_WHITE, COLOR_MAGENTA);
	
  
  let mut buffer = Vec::new();
  let size = File::open(env::args().nth(1).unwrap()).unwrap().read_to_end(&mut buffer);
  let mut info = get_elf_info(&buffer);
  
  let mut cursor = Cursor{index:0,length:buffer.len()*2+1,offset:0};

  loop {
    if cursor.update_offset((window.get_max_y()-2) as usize) {
      window.clear();
    }
    render(&window,&buffer,&info,&cursor);

    window.refresh();
    match window.getch() {
      Some(Input::Character('j'))=>{cursor.mv(0,-1);},
      Some(Input::KeyUp)         =>{cursor.mv(0,-1);},
      Some(Input::Character('k'))=>{cursor.mv(0,1);},
      Some(Input::KeyDown)       =>{cursor.mv(0,1);},
      Some(Input::Character('h'))=>{cursor.mv(-1,0);},
      Some(Input::KeyLeft)       =>{cursor.mv(-1,0);},
      Some(Input::Character('l'))=>{cursor.mv(1,0);},
      Some(Input::KeyRight)      =>{cursor.mv(1,0);},
      Some(Input::Character(other)) if other >= '0' && other <= '9' => {buffer.set_at((other as u8) -('0' as u8),&mut cursor); info = get_elf_info(&buffer);},
      Some(Input::Character(other)) if other >= 'a' && other <= 'f' => {buffer.set_at((other as u8) -('a' as u8)+10,&mut cursor); info = get_elf_info(&buffer);},
      Some(Input::Character('\u{1b}')) => break,
      Some(other)=>{info.msg = format!("Unused keypress: {:?}",other);},
      None => ()
    }
  }
  endwin();
}
trait setat {
  fn set_at(&mut self,u8,&mut Cursor);
}
impl setat for Vec<u8> {
  fn set_at(&mut self, val: u8, cursor: &mut Cursor){
    if cursor.index == cursor.length-1 {
      self.push(val*16); // move into the upper half of the new byte
      cursor.length += 2;
      cursor.index  += 1;
      return;
    }
    if cursor.index % 2 == 1 {
      self[cursor.index/2] = (self[cursor.index/2]&0xF0) + val;
    }else{
      self[cursor.index/2] = (self[cursor.index/2]&0x0F) + val*16;
    }
    cursor.index += 1;
  }
}

struct Cursor {
  index: usize,
  length: usize,
  offset: usize
}

impl Cursor {
  fn mv(&mut self, x:i32, y:i32){
    let delta = x + y*32;
    if delta > 0 {
      if self.index + delta as usize>= self.length {
        self.index = self.length-1
      } else {
        self.index += delta as usize;
      }
    } else {
      if self.index > (-delta) as usize {
        self.index -= (-delta) as usize;
      } else {
        self.index = 0;
      }
    }
  }
  fn x(&self) -> usize {
    return if self.index % 32 >= 16{1}else{0} + ((self.index)%32)/2*3+self.index%2 + 10;
  }
  fn y(&self) -> usize {
    return self.index/32;
  }
  fn update_offset(&mut self, height: usize) -> bool{
    if self.offset > self.y() || self.y() - self.offset > height{
      let offset = if self.y() < height/2 {
        0
      } else if self.length/32-self.y() < height/2 {
        if self.length/32 > height {self.length/32 - height} else {0}
      } else {
        self.y()-height/2
      };
      if offset != self.offset {
        self.offset = offset;
        return true;
      }
    }
    return false;
  }
}


fn render(window: &Window, buffer: &Vec<u8>, table: &ELFinfo, cursor:&Cursor){
  window.mv(0,0);
  for row in cursor.offset..cmp::min(buffer.len()/16+1,cursor.offset+(window.get_max_y()-1) as usize){
    window.printw(&format!("{:08X}: ",row*16));
    for i in 0..16 {
      if row*16+i >= buffer.len() {continue;}
      if i == 8 {window.printw(" ");}
      window.attrset(ColorPair(highlight(16*row+i,&table)));
      window.printw(&format!("{:02X}",buffer[row*16+i]));
      if highlight(16*row+i,&table) != highlight(16*row+i+1,&table) || i == 15 {
        window.attrset(ColorPair(0));
      }
      window.printw(" ");
    }
    window.printw("\n");
  }

  print_elf_info(window,&table,&buffer,cursor.offset as i32);
  
  window.mv((cursor.y()-cursor.offset) as i32,cursor.x() as i32);
}

fn get_elf_info(buffer: &Vec<u8>) -> ELFinfo {
  let mut tmp = ELFinfo{bit_class:buffer[4],endianess:buffer[5],version:buffer[6],abi:buffer[7],
         file_type:get_multibyte_data(&buffer[16..18],buffer[5]==1) as u16,
              arch:get_multibyte_data(&buffer[18..20],buffer[5]==1) as u16,
              entry:get_multibyte_data(&buffer[24..32],buffer[5]==1),
              prog_head:get_multibyte_data(&buffer[32..40],buffer[5]==1),
              sect_head:get_multibyte_data(&buffer[40..48],buffer[5]==1),
              flags:get_multibyte_data(&buffer[48..52],buffer[5]==1) as u32,
              size:get_multibyte_data(&buffer[52..54],buffer[5]==1) as u16,
              prog_size:get_multibyte_data(&buffer[54..56],buffer[5]==1) as u16,
              prog_num:get_multibyte_data(&buffer[56..58],buffer[5]==1) as u16,
              sect_size:get_multibyte_data(&buffer[58..60],buffer[5]==1) as u16,
              sect_num:get_multibyte_data(&buffer[60..62],buffer[5]==1) as u16,
              shs_table_index:get_multibyte_data(&buffer[62..64],buffer[5]==1) as u16,
              progs:Vec::new(),sects:Vec::new(),shst:STRTAB{offset:0,size:0},
              msg:"".to_owned()
  };
  if tmp.prog_head == 0 && tmp.prog_num != 0 {
    tmp.msg = "Program headers offset = 0, but there are program headers".to_owned();
    return tmp;
  }
  if tmp.sect_head == 0 && tmp.sect_num != 0 {
    tmp.msg = "Section headers offset = 0, but there are program headers".to_owned();
    return tmp;
  }
  if tmp.prog_head > buffer.len() as u64 || tmp.prog_head + (tmp.prog_size as u64)*(tmp.prog_num as u64) > buffer.len() as u64{
    tmp.msg = "Error parsing. Program header outside file.".to_owned();
    return tmp;
  }
  if tmp.sect_head > buffer.len() as u64 || tmp.sect_head + (tmp.sect_size as u64)*(tmp.sect_num as u64) > buffer.len() as u64{
    tmp.msg = "Error parsing. Section header outside file.".to_owned();
    return tmp;
  }
  for i in 0..tmp.prog_num {
    let offset = (tmp.prog_head+(tmp.prog_size as u64)*(i as u64)) as usize;
    tmp.progs.push(ProgHead{typ:get_multibyte_data(&buffer[offset..(offset+4)],tmp.endianess==1) as u32,
             flags:get_multibyte_data(&buffer[(offset+4)..(offset+8)],tmp.endianess==1) as u32,
             offset:get_multibyte_data(&buffer[(offset+8)..(offset+16)],tmp.endianess==1),
             virt_addr:get_multibyte_data(&buffer[(offset+16)..(offset+24)],tmp.endianess==1),
             file_size:get_multibyte_data(&buffer[(offset+32)..(offset+40)],tmp.endianess==1),
             mem_size:get_multibyte_data(&buffer[(offset+40)..(offset+48)],tmp.endianess==1),
             align:get_multibyte_data(&buffer[(offset+48)..(offset+56)],tmp.endianess==1)
    });
  }
  for i in 0..tmp.sect_num {
    let offset = (tmp.sect_head+(tmp.sect_size as u64)*(i as u64)) as usize;
    tmp.sects.push(SectHead{name:get_multibyte_data(&buffer[offset..(offset+4)],tmp.endianess==1) as u32,
             typ:get_multibyte_data(&buffer[(offset+4)..(offset+8)],tmp.endianess==1) as u32,
             flags:get_multibyte_data(&buffer[(offset+8)..(offset+16)],tmp.endianess==1),
             virt_addr:get_multibyte_data(&buffer[(offset+16)..(offset+24)],tmp.endianess==1),
             offset:get_multibyte_data(&buffer[(offset+24)..(offset+32)],tmp.endianess==1),
             file_size:get_multibyte_data(&buffer[(offset+32)..(offset+40)],tmp.endianess==1),
             sect_index:get_multibyte_data(&buffer[(offset+40)..(offset+44)],tmp.endianess==1) as u32,
             extra_info:get_multibyte_data(&buffer[(offset+44)..(offset+48)],tmp.endianess==1) as u32,
             align:get_multibyte_data(&buffer[(offset+48)..(offset+56)],tmp.endianess==1),
             entry_size:get_multibyte_data(&buffer[(offset+56)..(offset+64)],tmp.endianess==1),
    });
  }
  if (tmp.shs_table_index as usize) < tmp.sects.len(){
    let shs_head = &tmp.sects[tmp.shs_table_index as usize];
    if shs_head.typ == 3 {
      if shs_head.offset > buffer.len() as u64 || shs_head.offset + shs_head.file_size > buffer.len() as u64 {
        tmp.msg = "String table section not within file".to_owned();
      }
      tmp.shst.offset = shs_head.offset;
      tmp.shst.size = shs_head.file_size;
    } else {
      tmp.msg = "String table section header corrupted".to_owned();
    }
  }
  return tmp;
}

fn get_multibyte_data(data: &[u8], little_endian: bool) -> u64{
  let mut sum : u64 = 0;
  if little_endian {
    for i in 1..(data.len()+1) {
      sum = sum*256 + (data[data.len()-i] as u64);
    }
  } else {
    for i in 0..data.len() {
      sum = sum*256 + (data[i] as u64);
    }
  }
  return sum;
}

fn print_elf_info(window: &Window, info: &ELFinfo, buffer: &Vec<u8>, offset: i32){
  if offset == 0 {
    window.attrset(ColorPair(2));
    window.mvaddstr(0-offset,60,if info.bit_class == 1 {"32 bit"} else {"64 bit"});
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(3));
    window.printw(if info.endianess == 1 {"Little Endian"} else {"Big Endian"});
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(4));
    window.printw(&format!("Version: {}",info.version));
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(5));
    window.printw(match info.abi { 0 => "System V", 3 => "Linux", _ => "Unknown ABI"});
    window.attrset(ColorPair(6));
  }
  if offset <= 1 {
    window.mvaddstr(1-offset,60,match info.file_type { 1 => "T: Relocatable", 2 => "T: Executable", 3 => "T: Shared",
                    4 => "T: Core", _ => "Unknown type"});
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(2));
    window.printw(match info.arch {2 => "SPARC",3=>"x86",8=>"MIPS",0x14=>"PowerPC",0x28=>"ARM",0x2A=>"SuperH",0x32=>"IA-64",0x3E=>"0x86-64",0xB7=>"AArch67",
                                   0xF3=>"RISC-V",_=>"Unknown ISA"});
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(3));
    window.printw(&format!("Entry: 0x{:08X}",info.entry));
    window.attrset(ColorPair(5));
  }
  if offset <= 2 {
    window.mvaddstr(2-offset,60,&format!("&progs: 0x{:08X}",info.prog_head));
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(6));
    window.printw(&format!("&sects: 0x{:08X}",info.sect_head));
    window.attrset(ColorPair(1));
  }
  if offset <= 3 {
    window.mvaddstr(3-offset,60,&format!("Flags: 0x{:04X}",info.flags));
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(2));
    window.printw(&format!("S: {}b",info.size));
    window.attrset(ColorPair(0));
    window.printw(" progs:");
    window.attrset(ColorPair(4));
    window.printw(&format!("{}",info.prog_num));
    window.attrset(ColorPair(0));
    window.printw("x");
    window.attrset(ColorPair(3));
    window.printw(&format!("{}b",info.prog_size));
    window.attrset(ColorPair(0));
    window.printw(" sects:");
    window.attrset(ColorPair(6));
    window.printw(&format!("{}",info.sect_num));
    window.attrset(ColorPair(0));
    window.printw("x");
    window.attrset(ColorPair(5));
    window.printw(&format!("{}b",info.sect_size));
    window.attrset(ColorPair(0));
    window.printw(" ");
    window.attrset(ColorPair(1));
    window.printw(&format!("shsti: {} ",info.shs_table_index));
    window.attrset(ColorPair(0));
  }
  
  window.attrset(ColorPair(0));
  
  if info.shst.offset != 0{
    for i in info.shst.offset/16..((info.shst.offset+info.shst.size)/16+1){
      if i as i32 >= offset && (i as i32) - offset < window.get_max_y()-1 {
        window.mvaddstr(i as i32 - offset,60,"|");
        for k in 0..16 {
          let index = (k+i*16) as usize;
          window.printw(&format!("{}",if index < buffer.len() && buffer[index] >= 32 && buffer[index] < 127 {buffer[index] as char} else {'.'}));
        }
        window.printw("|");
      }
    }
  }
  
  
  for i in 0..(info.progs.len() as i32){
    let head = &info.progs[i as usize];
    let base = ((info.prog_head+((i as u64)*(info.prog_size as u64)))/16) as i32 - offset;
    
    if base >= 0 && base < window.get_max_y()-1{
      window.attrset(ColorPair(2));
      window.mvaddstr(base,60,match head.typ {0=>"Null",1=>"Load", 2=>"Dynamic",3=>"Interp",4=>"Note",5=>"SHLIB",
                      6=>"PHDR",0x60000000=>"LOOS",0x6FFFFFFF=>"HIOS",0x70000000=>"LOPROC",0x7FFFFFFF=>"HIPROC", _=>"Unknown type"});
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(3));
      window.printw(if head.flags & 4 == 0 {" "} else {"R"});
      window.printw(if head.flags & 2 == 0 {" "} else {"W"});
      window.printw(if head.flags & 1 == 0 {" "} else {"E"});
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(4));
      window.printw(&format!("Offset: 0x{:08X}",head.offset));
    }
    
    if base+1 >= 0 && base+1 < window.get_max_y()-1{
      window.attrset(ColorPair(5));
      window.mvaddstr(base+1,60,&format!("Virt Addr: 0x{:08X}",head.virt_addr));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(6));
      window.printw(&format!("Size in file: 0x{:08X}",head.file_size));
    }
    
    if base+2 >= 0 && base+2 < window.get_max_y()-1{
      window.attrset(ColorPair(1));
      window.mvaddstr(base+2,60,&format!("Size in mem: 0x{:08X}",head.mem_size));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(3));
      window.printw(&format!("alignment: 0x{:08X}",head.align));
    }
  }
  
  for i in 0..(info.sects.len() as i32){
    let head = &info.sects[i as usize];
    let base = ((info.sect_head+((i as u64)*(info.sect_size as u64)))/16) as i32 - offset;
    
    if base >= 0 && base < window.get_max_y()-1{
      window.attrset(ColorPair(1));
      window.mvaddstr(base,60,&format!("Name index: {}", head.name));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(2));
      window.printw(match head.typ {0=>"NULL",1=>"PROGBITS", 2=>"SYMTAB",3=>"STRTAB",4=>"RELA",5=>"HASH",
                          6=>"DYNAMIC",7=>"NOTE",8=>"NOBITS",9=>"REL",10=>"SHLIB",11=>"DYNSYM",0x60000000=>"LOOS",
                          0x6FFFFFFF=>"HIOS",0x70000000=>"LOPROC",0x7FFFFFFF=>"HIPROC", _=>"Unknown type"});
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(3));
      window.printw(if head.flags & 1 == 0 {" "} else {"R"});
      window.printw(if head.flags & 2 == 0 {" "} else {"A"});
      window.printw(if head.flags & 4 == 0 {" "} else {"E"});
    }
    
    if base+1 >= 0 && base+1 < window.get_max_y()-1{
      window.attrset(ColorPair(4));
      window.mvaddstr(base+1,60,&format!("Virt Addr: 0x{:08X}",head.virt_addr));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(5));
      window.printw(&format!("Offset: 0x{:08X}",head.offset));
    }
    
    if base+2 >= 0 && base+2 < window.get_max_y()-1{
      window.attrset(ColorPair(6));
      window.mvaddstr(base+2,60,&format!("Size in file: 0x{:08X}",head.file_size));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(1));
      window.printw(&format!("Section Index: {}",head.sect_index));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(2));
      window.printw(&format!("Extra info: 0x{:04X}",head.extra_info));
    }
    
    if base+3 >= 0 && base+3 < window.get_max_y()-1{
      window.attrset(ColorPair(3));
      window.mvaddstr(base+3,60,&format!("Alignment: 0x{:08X}",head.align));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(4));
      window.printw(&format!("Entry size: 0x{:08X}",head.entry_size));
    }
  }
  window.attrset(ColorPair(0));
  
  window.mvaddstr(window.get_max_y()-1,0,&info.msg);
  window.clrtoeol();
  
}

struct ELFinfo {
  bit_class: u8,
  endianess: u8,
  version: u8,
  abi: u8,
  file_type: u16,
  arch: u16,
  entry: u64,
  prog_head:u64,
  sect_head:u64,
  flags:u32,
  size:u16,
  prog_size:u16,
  prog_num: u16,
  sect_size:u16,
  sect_num: u16,
  shs_table_index:u16,
  progs:Vec<ProgHead>,
  sects:Vec<SectHead>,
  shst:STRTAB,
  msg:String
}
struct STRTAB {
  offset: u64,
  size: u64
}

struct ProgHead {
  typ: u32,
  flags: u32,
  offset: u64,
  virt_addr: u64,
  file_size: u64,
  mem_size: u64,
	align: u64
}

#[derive(Debug)]
struct SectHead {
  name: u32,
  typ: u32,
  flags: u64,
  virt_addr: u64,
  offset: u64,
  file_size: u64,
  sect_index: u32,
  extra_info: u32,
	align: u64,
	entry_size:u64
}

fn highlight(index: usize, info: &ELFinfo) -> u8{
	if index <= 3  {return 1} // Magic bytes
	if index == 4  {return 2} // Bittedness
	if index == 5  {return 3} // Endianess
	if index == 6  {return 4} // Version
        if index == 7  {return 5} // OS / ABI
	if index <  16 {return 0} // Extra shit and reserved
	if index <= 17 {return 6} // Type (EXEC)
	if index <= 19 {return 2} // Machine type
	if index <= 23 {return 4} // Version
	if index <  32 {return 3} // Entry point
	if index <  40 {return 5} // Start of Program headers
	if index <  48 {return 6} // Start of Section headers
	if index <= 51 {return 1} // Flags
	if index <= 53 {return 2} // Size of this header
	if index <= 55 {return 3} // Size of Program headers
	if index <= 57 {return 4} // Number of Program headers
	if index <= 59 {return 5} // Size of Section headers
	if index <= 61 {return 6} // Number of Section headers
	if index <= 63 {return 1} // Section header string table index
  for i in 0..info.prog_num {
    if index < (info.prog_head + (i as u64)*(info.prog_size as u64)) as usize {continue;}
    if index > (info.prog_head + ((i as u64)+1)*(info.prog_size as u64)) as usize{continue;}
    let offset = (index as u64) - info.prog_head - (i as u64)*(info.prog_size as u64);
    if offset <= 3 {return 2} // Type
    if offset <= 7 {return 3} // Flags
    if offset < 16 {return 4} // Offset
    if offset < 24 {return 5} // Virtual Address
    if offset < 32 {return 0} // Physical address / unused
    if offset < 40 {return 6} // Size in file
    if offset < 48 {return 1} // Size in Mem
    if offset < 56 {return 3} // Required alignment
  }
  for i in 0..info.sect_num {
    if index < (info.sect_head + (i as u64)*(info.sect_size as u64)) as usize {continue;}
    if index > (info.sect_head + ((i as u64)+1)*(info.sect_size as u64)) as usize{continue;}
    let offset = (index as u64) - info.sect_head - (i as u64)*(info.sect_size as u64);
    if offset <= 3 {return 1} // Name
    if offset <= 7 {return 2} // Type
    if offset < 16 {return 3} // Flags
    if offset < 24 {return 4} // Virtual Address
    if offset < 32 {return 5} // Address in file
    if offset < 40 {return 6} // Size in file
    if offset < 44 {return 1} // Section index
    if offset < 48 {return 2} // Extra info
    if offset < 56 {return 3} // Required alignment
    if offset < 64 {return 4} // Entry size
  }
	return 0;
}

