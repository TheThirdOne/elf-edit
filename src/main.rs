extern crate pancurses;

use std::fs::File;
use std::io::Read;
use pancurses::*;
use std::env;
use std::cmp;

mod data;
use data::*;
mod helpers;
use helpers::*;
mod render;
use render::*;

fn main() {
  let window = initscr();
  start_color();
  use_default_colors();
  
  noecho();

  window.keypad(true);
  init_pair(1, COLOR_BLACK, COLOR_YELLOW);
  init_pair(2, COLOR_BLACK, COLOR_GREEN);
  init_pair(3, COLOR_BLACK, COLOR_CYAN);
  init_pair(4, COLOR_BLACK, COLOR_BLUE);
  init_pair(5, COLOR_WHITE, COLOR_RED);
  init_pair(6, COLOR_WHITE, COLOR_MAGENTA);
	
  
  let mut buffer = Vec::new();
  let name = match env::args().nth(1) {
               Some(n)=>n,
               None => {
                  endwin();
                  println!("Usage: elfedit exec.o");
                  return;
                }
             };
  let _size = match File::open(name){
               Ok(f)=>f,
               Err(e)=>{
                 endwin();
                 println!("File couldn't be opened: {}", e);
                 return
               }
      }.read_to_end(&mut buffer);
  let mut info = get_elf_info(&buffer);
  
  let mut cursor = Cursor{index:0,length:buffer.len()*2+1,offset:0};

  loop {
    if cursor.update_offset((window.get_max_y()-2) as usize) {
      window.clear(); // maybe try something more exact than this (using insertln and deleteln)
      info.needs_redraw = true;
    }
    render(&window,&buffer,&info,&cursor);

    window.refresh();
    info.needs_redraw = false;
    match window.getch() {
      Some(Input::Character('k'))=>{cursor.mv(0,-1);},
      Some(Input::KeyUp)         =>{cursor.mv(0,-1);},
      Some(Input::Character('j'))=>{cursor.mv(0,1);},
      Some(Input::KeyDown)       =>{cursor.mv(0,1);},
      Some(Input::Character('h'))=>{cursor.mv(-1,0);},
      Some(Input::KeyLeft)       =>{cursor.mv(-1,0);},
      Some(Input::Character('l'))=>{cursor.mv(1,0);},
      Some(Input::KeyRight)      =>{cursor.mv(1,0);},
      Some(Input::Character(other)) if other >= '0' && other <= '9' => {buffer.set_at((other as u8) -('0' as u8),&mut cursor); info = get_elf_info(&buffer);},
      Some(Input::Character(other)) if other >= 'a' && other <= 'f' => {buffer.set_at((other as u8) -('a' as u8)+10,&mut cursor); info = get_elf_info(&buffer);},
      Some(Input::Character('\u{1b}')) => break,
      Some(other)=>{info.msg = format!("Unused keypress: {:?}",other);},
      None => (),
    };
  }
  endwin();
}

fn render(window: &Window, buffer: &Vec<u8>, table: &ELFinfo, cursor:&Cursor){
  window.mv(0,0);
  if table.needs_redraw {
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
  }
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
              symtab: SYMTAB {offset:0,size:0,entry_size:0,link:0,ei:0},
              progs:Vec::new(),sects:Vec::new(),shstr:STRTAB{offset:0,size:0},strtabs:Vec::new(),symbols:Vec::new(),reltabs:Vec::new(),
              msg:"".to_owned(),needs_redraw:true,
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
                            name_str:"".to_owned(),
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
    let i = tmp.shs_table_index as usize;
    if tmp.sects[i].typ == 3 {
      if tmp.sects[i].offset > buffer.len() as u64 {
        tmp.msg = "String table section not within file".to_owned();
        return tmp;
      } else if tmp.sects[i].offset > buffer.len() as u64 {
        tmp.msg = "String table not entirely within file".to_owned();
        return tmp;
      }
      tmp.shstr.offset = tmp.sects[i].offset;
      tmp.shstr.size = tmp.sects[i].file_size;
    } else {
      tmp.msg = "String table section header corrupted".to_owned();
      return tmp;
    }
  }
  for prog in &mut tmp.progs {
    if prog.typ == 3 { //Interp
      if prog.offset <= 0 || prog.offset > buffer.len() as u64 || prog.offset + prog.file_size > buffer.len() as u64 {
        tmp.msg = "String for INTERP not within file".to_owned();
      } else {
        tmp.strtabs.push(STRTAB{offset:prog.offset,size:prog.file_size});
      }
    }
  }
  for sect in &mut tmp.sects {
    sect.name_str = get_null_string(&buffer[(tmp.shstr.offset as usize)..((tmp.shstr.offset+tmp.shstr.size) as usize)],sect.name as usize);
    if sect.typ == 3 { //String table
      if sect.offset <= 0 || sect.offset > buffer.len() as u64 || sect.offset + sect.file_size > buffer.len() as u64 {
        tmp.msg = "String table section not within file".to_owned();
      } else {
        tmp.strtabs.push(STRTAB{offset:sect.offset,size:sect.file_size});
      }
    }else if sect.typ == 2 {
      tmp.symtab.offset = sect.offset;
      tmp.symtab.size = sect.file_size;
      tmp.symtab.entry_size = sect.entry_size;
      tmp.symtab.link = sect.sect_index;
      tmp.symtab.ei = sect.extra_info;
    } else if sect.typ == 4 {
      tmp.reltabs.push(RELTAB{offset:sect.offset,size:sect.file_size,entry_size:sect.entry_size,link:sect.sect_index,ei:sect.extra_info,rels:Vec::new()})
    }
  }
  
  for tab in &mut tmp.reltabs {
    if tab.offset + tab.size <= buffer.len() as u64 && tab.entry_size != 0 {
      for i in 0..(tab.size/tab.entry_size){
        let offset = ( tab.offset+(tab.entry_size as u64)*(i as u64)) as usize;
        tab.rels.push(REL{offset:get_multibyte_data(&buffer[offset..(offset+8)],tmp.endianess==1) as u64,
                 info:get_multibyte_data(&buffer[(offset+8)..(offset+16)],tmp.endianess==1) as u64,
                 addend:get_multibyte_data(&buffer[(offset+16)..(offset+24)],tmp.endianess==1) as i64});
      }
    }
  }
  
  if tmp.symtab.offset + tmp.symtab.size < buffer.len() as u64 && tmp.symtab.entry_size != 0{
    let strtab = &tmp.sects[tmp.symtab.link as usize];
    for i in 0..(tmp.symtab.size/tmp.symtab.entry_size){
      let offset = (tmp.symtab.offset+(tmp.symtab.entry_size as u64)*(i as u64)) as usize;
      let mut sym = SYMBOL{name:get_multibyte_data(&buffer[offset..(offset+4)],tmp.endianess==1) as u32,
               name_str:"".to_owned(),
               info:buffer[offset+4],other:buffer[offset+5],
               shndx:get_multibyte_data(&buffer[(offset+6)..(offset+8)],tmp.endianess==1) as u16,
               value:get_multibyte_data(&buffer[(offset+8)..(offset+16)],tmp.endianess==1) as u64,
               size:get_multibyte_data(&buffer[(offset+16)..(offset+24)],tmp.endianess==1) as u64};
      sym.name_str = get_null_string(&buffer[(strtab.offset as usize)..((strtab.offset+strtab.file_size) as usize)],sym.name as usize);
      tmp.symbols.push(sym);
    }
  }
  return tmp;
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
  for i in 0..info.progs.len() {
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
  for i in 0..info.sects.len() {
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
  for i in 0..info.symbols.len() {
    if index < (info.symtab.offset + (i as u64)*(info.symtab.entry_size as u64)) as usize {continue;}
    if index > (info.symtab.offset + (i as u64+1)*(info.symtab.entry_size as u64)) as usize{continue;}
    let offset = (index as u64) - info.symtab.offset - (i as u64)*(info.symtab.entry_size as u64);
    if offset < 4  {return 1} // Name
    if offset < 5  {return 2} // Info
    if offset < 6  {return 3} // Other / Visibility
    if offset < 8  {return 4} // Section index
    if offset < 16 {return 5} // Value
    if offset < 24 {return 6} // Size
  }
  for tab in &info.reltabs{
    for i in 0..tab.rels.len(){
      if index < (tab.offset + (i as u64)*(tab.entry_size as u64)) as usize {continue;}
      if index > (tab.offset + (i as u64+1)*(tab.entry_size as u64)) as usize{continue;}
      let offset = (index as u64) - tab.offset - (i as u64)*(tab.entry_size as u64);
      if offset < 8  {return 1} // Offset
      if info.arch == 0x3E {
        if offset < 12 {return 2} // Symbol index
        if offset < 16 {return 4} // Type
      } else {
        if offset < 16 {return 2} // Info
      } 
      if offset < 24 {return 3} // Addend
    }
  }
	return 0;
}
