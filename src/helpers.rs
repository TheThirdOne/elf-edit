pub fn get_multibyte_data(data: &[u8], little_endian: bool) -> u64{
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
  sum
}

pub fn get_null_string(data: &[u8],mut i: usize) -> String{
  let mut tmp = "".to_owned();
  while i < data.len() && data[i] != 0 && i < data.len(){
    tmp.push(data[i] as char);
    i += 1;
  }
  tmp
}

pub trait SetAt {
  fn set_at(&mut self,u8,&mut Cursor);
}
impl SetAt for Vec<u8> {
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

pub struct Cursor {
  pub index: usize,
  pub length: usize,
  pub offset: usize
}

impl Cursor {
  pub fn mv(&mut self, x:i32, y:i32){
    let delta = x + y*32;
    if delta > 0 {
      if self.index + delta as usize>= self.length {
        self.index = self.length-1
      } else {
        self.index += delta as usize;
      }
    }
    if self.index > (-delta) as usize {
      self.index -= (-delta) as usize;
    } else {
      self.index = 0;
    }
  }
  pub fn x(&self) -> usize {
    (if self.index % 32 >= 16{1}else{0} + ((self.index)%32)/2*3+self.index%2 + 10)
  }
  pub fn y(&self) -> usize {
    self.index/32
  }
  pub fn update_offset(&mut self, height: usize) -> bool{
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
    false
  }
}
