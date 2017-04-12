use pancurses::*;
use data::*;

pub fn print_elf_info(window: &Window, info: &ELFinfo, buffer: &Vec<u8>, offset: i32){
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
    window.printw(&format!("shstri: {} ",info.shs_table_index));
    window.attrset(ColorPair(0));
  }
  
  window.attrset(ColorPair(0));
  
  for tab in &info.strtabs {
    for i in tab.offset/16..((tab.offset+tab.size)/16+1){
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
      window.mvaddstr(base,60,&format!("Name: {} ({})", &head.name_str, head.name));
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
  
  for i in 0..(info.symbols.len() as i32){
    let head = &info.symbols[i as usize];
    let base = ((info.symtab.offset+((i as u64)*(info.symtab.entry_size as u64)))/16) as i32 - offset;
    
    if base >= 0 && base < window.get_max_y()-1{
      window.attrset(ColorPair(1));
      window.mvaddstr(base,60,&format!("Name: {} ({})", &head.name_str, head.name));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(2));
      window.printw(&format!("Bind: {}, Type: {}",match head.info >> 4 {0=>"Local",1=>"Global",2=>"Weak",10=>"LOOS",12=>"HIOS",13=>"LOPROC",15=>"HIPROC",_=>"Unknown type"},
      match head.info & 0xf {0=>"No Type",1=>"Object",2=>"Func",3=>"Section",4=>"File",5=>"Common",6=>"TLS",10=>"LOOS",12=>"HIOS",13=>"LOPROC",14=>"Spark Register",15=>"HIPROC",_=>"Unknown type"}));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(3));
      window.printw(&format!("Visibility: {}",match head.other & 0x3 {0=>"Default",1=>"Internal",2=>"Hidden",3=>"Protected",4=>"Exported",5=>"Singleton",6=>"Eliminate",_=>"Unknown visbility"}));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(4));
      let str = match head.shndx {0=>"Undef",0xff00=>"LOPROC",0xff01=>"After",0xff1f=>"HIPROC",0xff20=>"LOOS",0xff3f=>"HIOS",
                                  0xfff1=>"ABS",0xfff2=>"Common",0xffff=>"HIReserve", _=>""};
      if str == ""{
        window.printw(&format!("Shndx: {}",head.shndx));
      }else{
        window.printw(&format!("Shndx: {}",str));
      }
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(5));
      window.printw(&format!("Value: 0x{:04X}",head.value));
      window.attrset(ColorPair(0));
      window.printw(" ");
      window.attrset(ColorPair(6));
      window.printw(&format!("Size: 0x{:04X}",head.size));
    }
  }
  
  for tab in &info.reltabs{
    for i in 0..tab.rels.len(){
      let base = ((tab.offset + (i as u64)*(tab.entry_size as u64))/16) as i32 - offset;
      if base >= 0 && base < window.get_max_y()-1{
        window.attrset(ColorPair(1));
        window.mvaddstr(base,60,&format!("Offset: 0x{:04x}", tab.rels[i].offset));
        window.attrset(ColorPair(0));
        window.printw(" ");
        window.attrset(ColorPair(2));
        window.printw(&format!("Info: 0x{:08x}", tab.rels[i].info));
        window.attrset(ColorPair(0));
        window.printw(" ");
        window.attrset(ColorPair(3));
        window.printw(&format!("Addend: {}", tab.rels[i].addend));
      }
    }
  }
  
  window.attrset(ColorPair(0));
  
  window.mvaddstr(window.get_max_y()-1,0,&info.msg);
  window.clrtoeol();
}

