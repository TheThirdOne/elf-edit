pub struct ELFinfo {
  pub bit_class: u8,
  pub endianess: u8,
  pub version: u8,
  pub abi: u8,
  pub file_type: u16,
  pub arch: u16,
  pub entry: u64,
  pub prog_head:u64,
  pub sect_head:u64,
  pub flags:u32,
  pub size:u16,
  pub prog_size:u16,
  pub prog_num: u16,
  pub sect_size:u16,
  pub sect_num: u16,
  pub shs_table_index:u16,
  pub progs:Vec<ProgHead>,
  pub sects:Vec<SectHead>,
  pub shstr:STRTAB,
  pub strtabs:Vec<STRTAB>,
  pub symtab: SYMTAB,
  pub symbols:Vec<SYMBOL>,
  pub msg:String
}
pub struct STRTAB {
  pub offset: u64,
  pub size: u64
}

pub struct SYMTAB {
  pub offset: u64,
  pub size:   u64,
  pub entry_size: u64,
  pub link: u32,
  pub ei:   u32
}

pub struct SYMBOL {
  pub name:  u32,
  pub name_str:String,
  pub info:  u8,
  pub other: u8, //used for visibility
  pub shndx: u16,
  pub value: u64,
  pub size:  u64
}

pub struct ProgHead {
  pub typ: u32,
  pub flags: u32,
  pub offset: u64,
  pub virt_addr: u64,
  pub file_size: u64,
  pub mem_size: u64,
	pub align: u64
}

pub struct SectHead {
  pub name: u32,
  pub name_str: String,
  pub typ: u32,
  pub flags: u64,
  pub virt_addr: u64,
  pub offset: u64,
  pub file_size: u64,
  pub sect_index: u32,
  pub extra_info: u32,
	pub align: u64,
	pub entry_size:u64
}