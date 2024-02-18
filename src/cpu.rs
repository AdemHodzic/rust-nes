mod cpu_constants {
    pub const LDA_IMMEDIATE: u8 = 0xA9;
    pub const LDA_ZP: u8 = 0xA5;
    pub const LDA_ZPX: u8 = 0xB5;
    pub const LDA_ABS: u8 = 0xAD;
    pub const LDA_ABSX: u8 = 0xBD;
    pub const LDA_ABSY: u8 = 0xB9;
    pub const LDA_INDX: u8 = 0xA1;
    pub const LDA_INDY: u8 = 0xB1;

    pub const STA_ZP: u8 = 0x85;

    pub const TAX: u8 = 0xAA;
    pub const TAY: u8 = 0xA8;
    pub const INX: u8 = 0xE8;
    pub const BRK: u8 = 0x00;
}


pub mod cpu {


    pub struct CPU {
        pub register_a: u8,
        pub status: u8,
        pub program_counter: u16,

        pub register_x: u8,
        pub register_y: u8,
        memory: [u8; 0xFFFF]
    }
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum AddressingMode {
        Immediate,
        ZeroPage,
        ZeroPage_X,
        ZeroPage_Y,
        Absolute,
        Absolute_X,
        Absolute_Y,
        Indirect_X,
        Indirect_Y,
        NoneAddressing,
    }
    
    pub struct OpCode {
        pub opcode: u8,
        pub name: &'static str,
        pub takes_bytes: u16,
        pub takes_cycles: u16,
        pub adressing_mode: AddressingMode
    }

    impl OpCode {
        pub fn new(
            opcode: u8,
            name: &'static str,
            takes_bytes: u16,
            takes_cycles: u16,
            adressing_mode: AddressingMode
        ) -> Self {
            OpCode {
                opcode: opcode,
                name: name,
                takes_bytes: takes_bytes,
                takes_cycles: takes_cycles,
                adressing_mode: adressing_mode,
            }
        }
    }

    lazy_static! {
        pub static ref OPCODES: Vec<OpCode> = vec![
            // LDA
            OpCode::new(0xA9, "LDA", 2,2,AddressingMode::Immediate),
            OpCode::new(0xA5, "LDA", 2,3,AddressingMode::ZeroPage),
            OpCode::new(0xB5, "LDA", 2,4,AddressingMode::ZeroPage_X),
            OpCode::new(0xAD, "LDA", 2,4,AddressingMode::Absolute),
            OpCode::new(0xBD, "LDA", 2,4,AddressingMode::Absolute_X),
            OpCode::new(0xB9, "LDA", 2,4,AddressingMode::Absolute_Y),
            OpCode::new(0xA1, "LDA", 2,6,AddressingMode::Indirect_X),
            OpCode::new(0xB1, "LDA", 2,5,AddressingMode::Indirect_Y),

            // STA
            OpCode::new(0x85, "STA", 2,3,AddressingMode::ZeroPage),
            OpCode::new(0x95, "STA", 2,4,AddressingMode::ZeroPage_X),
            OpCode::new(0x8D, "STA", 3,4,AddressingMode::Absolute),
            OpCode::new(0x9D, "STA", 3,5,AddressingMode::Absolute_X),
            OpCode::new(0x99, "STA", 3,5,AddressingMode::Absolute_Y),
            OpCode::new(0x81, "STA", 2,6,AddressingMode::Indirect_X),
            OpCode::new(0x91, "STA", 2,6,AddressingMode::Indirect_Y),

            // TAX
            OpCode::new(0xAA, "TAX", 1,2,AddressingMode::NoneAddressing),
            // TAY
            OpCode::new(0xA8, "TAY", 1,2,AddressingMode::NoneAddressing),
            
            //INX
            OpCode::new(0xE8, "INX", 1,2,AddressingMode::NoneAddressing),
            //BRK
            OpCode::new(0x00, "BRK", 1,7,AddressingMode::NoneAddressing),
            
        ];
    }
    
    pub fn find_opcode_by_instruction(instruction: u8) -> Option<&'static OpCode> {
        for opcode in OPCODES.iter() {
            if opcode.opcode == instruction {
                return Some(opcode);
            }
        }

        None
    }

    impl CPU {
        pub fn new() -> Self {
            CPU {
                register_a: 0,
                status: 0,
                program_counter: 0,

                register_x: 0,
                register_y: 0,

                memory: [0; 0xFFFF]

            }
        }

        
            
        pub fn mem_read(&self, addr: u16) -> u8 {
            self.memory[addr as usize]
        }

        fn mem_read_u16(&mut self, pos: u16) -> u16 {
            let lo = self.mem_read(pos) as u16;
            let hi = self.mem_read(pos + 1) as u16;
            (hi << 8) | (lo as u16)
        }
    
        pub fn mem_write(&mut self, addr: u16, data: u8) {
            self.memory[addr as usize] = data;
        }

        fn mem_write_u16(&mut self, pos: u16, data: u16) {
            let hi = (data >> 8) as u8;
            let lo = (data & 0xff) as u8;
            self.mem_write(pos, lo);
            self.mem_write(pos + 1, hi);
        }
     
        
        pub fn reset(&mut self) {
            self.register_a = 0;
            self.register_x = 0;
            self.status = 0;
     
            self.program_counter = self.mem_read_u16(0xFFFC);
        }
     
        pub fn load(&mut self, program: Vec<u8>) {
            self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
            self.mem_write_u16(0xFFFC, 0x8000);
        }
     
        pub fn load_and_run(&mut self, program: Vec<u8>) {
            self.load(program);
            self.reset();
            self.run()
        }
     
        fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

            match mode {
                AddressingMode::Immediate => self.program_counter,
     
                AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
               
                AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
             
                AddressingMode::ZeroPage_X => {
                    let pos = self.mem_read(self.program_counter);
                    let addr = pos.wrapping_add(self.register_x) as u16;
                    addr
                }
                AddressingMode::ZeroPage_Y => {
                    let pos = self.mem_read(self.program_counter);
                    let addr = pos.wrapping_add(self.register_y) as u16;
                    addr
                }
     
                AddressingMode::Absolute_X => {
                    let base = self.mem_read_u16(self.program_counter);
                    let addr = base.wrapping_add(self.register_x as u16);
                    addr
                }
                AddressingMode::Absolute_Y => {
                    let base = self.mem_read_u16(self.program_counter);
                    let addr = base.wrapping_add(self.register_y as u16);
                    addr
                }
     
                AddressingMode::Indirect_X => {
                    let base = self.mem_read(self.program_counter);
     
                    let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                    let lo = self.mem_read(ptr as u16);
                    let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                    (hi as u16) << 8 | (lo as u16)
                }
                AddressingMode::Indirect_Y => {
                    let base = self.mem_read(self.program_counter);
     
                    let lo = self.mem_read(base as u16);
                    let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                    let deref_base = (hi as u16) << 8 | (lo as u16);
                    let deref = deref_base.wrapping_add(self.register_y as u16);
                    deref
                }
              
                AddressingMode::NoneAddressing => {
                    panic!("mode {:?} is not supported", mode);
                }
            }
     
        }


        fn lda(&mut self, mode: &AddressingMode) {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
           
            self.register_a = value;
            self.update_zero_and_negative_flags(self.register_a);
        }
      
        fn sta(&mut self, mode: &AddressingMode) {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, self.register_a);
        }

        fn tax(&mut self) {
            self.register_x = self.register_a;
            self.update_zero_and_negative_flags(self.register_x);
        }
        fn tay(&mut self) {
            self.register_y = self.register_a;
            self.update_zero_and_negative_flags(self.register_x);
        }
        


        fn inx(&mut self) {
            self.register_x = self.register_x.wrapping_add(1);
            self.update_zero_and_negative_flags(self.register_x);
        }

        fn update_zero_and_negative_flags(&mut self, result: u8) {
             if result == 0 {
                 self.status = self.status | 0b0000_0010;
             } else {
                 self.status = self.status & 0b1111_1101;
             }
     
             if result & 0b1000_0000 != 0 {
                 self.status = self.status | 0b1000_0000;
             } else {
                 self.status = self.status & 0b0111_1111;
             }
         }

        pub fn run(&mut self) {

            loop {
                let instruction = self.mem_read(self.program_counter);
                let op_code = find_opcode_by_instruction(instruction).unwrap();
                self.program_counter += 1;
                

                match op_code.name  {
                    // LDA
                    "LDA" => {
                        self.lda(&op_code.adressing_mode);
                        self.program_counter += op_code.takes_bytes - 1;
                    }
                    // STA
                    "STA" => {
                        self.sta(&op_code.adressing_mode);
                        self.program_counter += op_code.takes_bytes - 1;
                    }
                    "TAX" => {
                        self.tax()
                    }
                    "TAY" => {
                        self.tay()
                    }
                    "INX" => {
                        self.inx()
                    }
                    "BRK" => {
                        return;
                    }
                    _ => todo!()
                }
            }
        }
    }

}



#[cfg(test)]
mod test {
   use crate::cpu::{cpu::*, cpu_constants::*};
 
    fn endify(lo: u8, hi: u8) -> u16 {
        ((hi << 8 ) | lo) as u16
    }

    fn dendify(bit: u16) -> (u8, u8) {
        let hi = (bit >> 8) as u8;
        let lo = (bit & 0xff) as u8;
        (lo, hi)
    }

   #[test]
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![LDA_IMMEDIATE, 0x05, BRK]);
       assert_eq!(cpu.register_a, 0x05);
       assert!(cpu.status & 0b0000_0010 == 0b00);
       assert!(cpu.status & 0b1000_0000 == 0);
   }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![LDA_IMMEDIATE, 0x00, BRK]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_lda_zero_page_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
 
        cpu.load_and_run(vec![LDA_ZP, 0x10, BRK]);
 
        assert_eq!(cpu.register_a, 0x55);
    }


    #[test]
    fn test_lda_zero_page_x_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10 + 0x0F, 0x55);
 
        cpu.load_and_run(vec![LDA_IMMEDIATE, 0x0F, TAX, LDA_ZPX, 0x10, BRK]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_absolute() {
        let mut cpu = CPU::new();
        cpu.mem_write(0xAAAA, 0x55);
 
        let (lo, hi) = dendify(0xAAAA);

        cpu.load_and_run(vec![LDA_ABS, lo, hi, BRK]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_absolute_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0xAAAA + 0x80, 0x55);
 
        let (lo, hi) = dendify(0xAAAA);

        cpu.load_and_run(vec![LDA_IMMEDIATE, 0x80, TAX, LDA_ABSX, lo, hi, BRK]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_absolute_y() {
        let mut cpu = CPU::new();
        cpu.mem_write(0xAAAA + 0x80, 0x55);
 
        let (lo, hi) = dendify(0xAAAA);

        cpu.load_and_run(vec![LDA_IMMEDIATE, 0x80, TAY, LDA_ABSY, lo, hi, BRK]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
   fn test_0xaa_tax_move_a_to_x() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![LDA_IMMEDIATE, 0x0A, TAX, BRK]);
 
       assert_eq!(cpu.register_x, 0x0A)
   }

   #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![LDA_IMMEDIATE, 0xc0, TAX, INX, BRK]);
 
       assert_eq!(cpu.register_x, 0xc1)
   }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![LDA_IMMEDIATE, 0xFF, TAX, INX, INX, BRK]);
        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![LDA_IMMEDIATE, 0xFF, STA_ZP, 0x16, BRK]);
        assert_eq!(cpu.mem_read(0x16), 0xFF);
    }
}