mod csr;
mod decode;
mod handle_trap;
mod privilege_mode;
mod rv32i;
mod rv32m;
mod rv32c;
mod zicsr;

#[cfg(test)]
mod interrupt;
mod instructions;

use super::bus;
use csr::Csr;
use privilege_mode::PrivilegeMode;
use crate::cpu::instructions::Instruction;

#[derive(Debug)]
pub enum StepResult {
    Ok(u32),
    Trap(u32),
    Jumped,
}

/// 4KB ページ
const PAGE_SIZE: usize = 4096;

/// RISC-V C 拡張を考慮して 2 byte ごとに命令を格納する
const ENTRY_COUNT: usize = PAGE_SIZE / 2;

/// 命令キャッシュのページ定義
type InstructionCachePage = [Instruction; ENTRY_COUNT];

/// CPU の内部状態
pub struct Cpu {
    /// 32本の汎用レジスタ (x0-x31)
    pub regs: [u32; 32],

    /// プログラムカウンタ
    pub pc: u32,

    /// 制御ステータスレジスタ (CSR)
    pub csr: Csr,

    /// 特権モード
    pub mode: PrivilegeMode,

    /// 命令キャッシュ
    pages: Vec<Option<InstructionCachePage>>,

    /// 現在参照している命令キャッシュのページ番号
    current_page_num: u32,
}

impl Cpu {
    pub fn new(pc: u32) -> Self {
        Self {
            regs: [0; 32],
            pc,
            csr: Csr::default(),
            mode: PrivilegeMode::Machine,
            pages: vec![None; 1024],
            current_page_num: 0xffffffff, // 最初は必ずキャッシュミスするように
        }
    }

    /// 1ステップ実行
    pub fn step<B: bus::Bus>(&mut self, bus: &mut B) -> StepResult {
        // クロックを進める
        bus.tick();

        // 実行前に割り込みをチェック
        if let Some(interrupt_code) = self.check_interrupts(bus) {
            let result = self.handle_trap(interrupt_code, 0);
            self.regs[0] = 0; // レジスタ 0 は常に 0 に保つ.
            self.current_page_num = 0xffffffff;
            return result;
        }

        let inst = self.fetch(bus);
        let result = self.exec(inst, bus);
        self.regs[0] = 0; // レジスタ 0 は常に 0 に保つ.

        match result {
            StepResult::Ok(inst_size) => {
                self.pc += inst_size;
                StepResult::Ok(inst_size)
            }
            StepResult::Jumped => {
                self.current_page_num = 0xffffffff;
                StepResult::Jumped
            }
            StepResult::Trap(code) => {
                let mtval = if code == 2 {
                    // 違法命令の場合、現在 pc が指している命令を mtval に格納
                    bus.read32(self.pc)
                } else {
                    0
                };
                self.handle_trap(code, mtval);
                self.current_page_num = 0xffffffff; // Trap handles PC change
                StepResult::Trap(code)
            }
        }
    }

    /// レジスタの状態をダンプ
    pub fn dump_registers(&self) {
        for (i, reg) in self.regs.iter().enumerate() {
            println!("x{:02}: 0x{:08x}", i, reg);
        }
        println!("pc : 0x{:08x}", self.pc);
        println!("mstatus: 0x{:08x}", self.csr.mstatus);
        println!("mtvec  : 0x{:08x}", self.csr.mtvec);
        println!("mepc   : 0x{:08x}", self.csr.mepc);
        println!("mcause : 0x{:08x}", self.csr.mcause);
        println!("mtval  : 0x{:08x}", self.csr.mtval);
    }

    /// PLIC から割り込みを取得する (Claim)
    pub fn claim_interrupt<B: bus::Bus>(&mut self, bus: &mut B) -> u32 {
        bus.plic_claim()
    }

    /// PLIC に割り込み完了を通知する (Complete)
    pub fn complete_interrupt<B: bus::Bus>(&mut self, bus: &mut B, source_id: u32) {
        bus.plic_complete(source_id);
    }

    /// 割り込みのチェックを行い、発生すべき割り込みがあればその例外コードを返す
    fn check_interrupts<B: bus::Bus>(&mut self, bus: &B) -> Option<u32> {
        // mstatus.MIE が 0 の場合は割り込みを受け付けない
        if (self.csr.mstatus & (1 << 3)) == 0 {
            return None;
        }

        // PLIC 等の外部信号を mip.MEIP に反映させる (最小構成として)
        if bus.get_interrupt_level() {
            self.csr.mip |= 1 << 11; // MEIP
        } else {
            // 注意: 本来は Claim 時に PLIC が CPU の MEIP を下げるという副作用があるが、
            // 現在の get_interrupt_level() 方式でも、Claim 後は最高優先度が
            // threshold を下回る（あるいは 0 になる）ため、MEIP が下げられる挙動は再現される。
            self.csr.mip &= !(1 << 11);
        }

        // タイマー割り込み信号を mip.MTIP に反映させる
        if bus.get_timer_interrupt_level() {
            self.csr.mip |= 1 << 7; // MTIP
        } else {
            self.csr.mip &= !(1 << 7);
        }

        // ソフトウェア割り込み信号を mip.MSIP に反映させる
        if bus.get_software_interrupt_level() {
            self.csr.mip |= 1 << 3; // MSIP
        } else {
            self.csr.mip &= !(1 << 3);
        }

        // mip と mie の論理積をとる
        let pending_interrupts = self.csr.mip & self.csr.mie;

        if pending_interrupts == 0 {
            return None;
        }

        // 優先順位: 外部割り込み > ソフトウェア割り込み > タイマー割り込み
        // 外部割り込み (Machine External Interrupt)
        if (pending_interrupts & (1 << 11)) != 0 {
            return Some(0x8000_000b); // MSB=1, Code=11
        }

        // ソフトウェア割り込み (Machine Software Interrupt)
        if (pending_interrupts & (1 << 3)) != 0 {
            return Some(0x8000_0003); // MSB=1, Code=3
        }

        // タイマー割り込み (Machine Timer Interrupt)
        if (pending_interrupts & (1 << 7)) != 0 {
            return Some(0x8000_0007); // MSB=1, Code=7
        }

        None
    }

    #[inline(always)]
    fn fetch<B: bus::Bus>(&mut self, bus: &mut B) -> Instruction {
        let page_num = (self.pc / PAGE_SIZE as u32) as usize;
        if page_num != self.current_page_num as usize {
            if page_num >= self.pages.len() {
                self.pages.resize(page_num + 1, None);
            }
            if self.pages[page_num].is_none() {
                self.current_page_num = page_num as u32;
                self.gen_cache_page(bus);
            } else {
                self.current_page_num = page_num as u32;
            }
        }
        let page_offset = (self.pc % PAGE_SIZE as u32) as usize;
        let inst = self.pages[page_num].unwrap()[page_offset / 2];
        if matches!(inst, Instruction::None) {
            self.gen_cache_page(bus);
            self.pages[page_num].unwrap()[page_offset / 2]
        } else {
            inst
        }
    }

    fn gen_cache_page<B: bus::Bus>(&mut self, bus: &mut B) {
        println!("Generating cache page for PC={:#x}", self.pc);
        let page_size = PAGE_SIZE as u32;
        let start_pc = self.pc & !(page_size - 1);
        let mut cache = [Instruction::None; ENTRY_COUNT];

        let mut raw_ptr = start_pc;
        while (raw_ptr % page_size) < page_size - 1 {
            let entry_idx = ((raw_ptr % page_size) / 2) as usize;
            if entry_idx >= ENTRY_COUNT { break; }

            // ページ境界を跨ぐ命令のチェック
            if (raw_ptr % page_size) == page_size - 2 {
                let inst_low = bus.read16(raw_ptr);
                let quadrant = Instruction::decode_quadrant(inst_low);
                if quadrant == 0b11 {
                    let inst_high = bus.read16(raw_ptr + 2);
                    let inst_bin = ((inst_high as u32) << 16) | inst_low as u32;
                    let (inst, _) = self.gen_inst_from_bin(inst_bin, quadrant);
                    cache[entry_idx] = inst;
                    break;
                }
            }

            let (inst, inst_size) = self.gen_inst(raw_ptr, bus);
            cache[entry_idx] = inst;
            raw_ptr += inst_size;
            if (raw_ptr % page_size) == 0 { break; }
        }
        self.pages[(start_pc / page_size) as usize] = Some(cache);
    }

    #[inline(always)]
    fn gen_inst_from_bin(&self, inst_bin: u32, quadrant: u16) -> (Instruction, u32) {
        let inst = match quadrant {
            0b11 => match Instruction::decode_opcode(inst_bin) {
                0b0110111 => Instruction::lui(inst_bin),
                0b0010111 => Instruction::auipc(inst_bin),
                0b1101111 => Instruction::jal(inst_bin),
                0b1100111 => Instruction::jalr(inst_bin),
                0b0010011 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => Instruction::addi(inst_bin),
                    0b001 => Instruction::slli(inst_bin),
                    0b010 => Instruction::slti(inst_bin),
                    0b011 => Instruction::sltiu(inst_bin),
                    0b100 => Instruction::xori(inst_bin),
                    0b101 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::srli(inst_bin),
                        0b0100000 => Instruction::srai(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b110 => Instruction::ori(inst_bin),
                    0b111 => Instruction::andi(inst_bin),
                    _ => Instruction::Illegal,
                },
                0b1100011 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => Instruction::beq(inst_bin),
                    0b001 => Instruction::bne(inst_bin),
                    0b100 => Instruction::blt(inst_bin),
                    0b101 => Instruction::bge(inst_bin),
                    0b110 => Instruction::bltu(inst_bin),
                    0b111 => Instruction::bgeu(inst_bin),
                    _ => Instruction::Illegal,
                },
                0b0000011 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => Instruction::lb(inst_bin),
                    0b001 => Instruction::lh(inst_bin),
                    0b010 => Instruction::lw(inst_bin),
                    0b100 => Instruction::lbu(inst_bin),
                    0b101 => Instruction::lhu(inst_bin),
                    _ => Instruction::Illegal,
                },
                0b0100011 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => Instruction::sb(inst_bin),
                    0b001 => Instruction::sh(inst_bin),
                    0b010 => Instruction::sw(inst_bin),
                    _ => Instruction::Illegal,
                },
                0b0110011 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::add(inst_bin),
                        0b0100000 => Instruction::sub(inst_bin),
                        0b0000001 => Instruction::mul(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b001 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::sll(inst_bin),
                        0b0000001 => Instruction::mulh(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b010 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::slt(inst_bin),
                        0b0000001 => Instruction::mulhsu(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b011 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::sltu(inst_bin),
                        0b0000001 => Instruction::mulhu(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b100 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::xor(inst_bin),
                        0b0000001 => Instruction::div(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b101 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::srl(inst_bin),
                        0b0100000 => Instruction::sra(inst_bin),
                        0b0000001 => Instruction::divu(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b110 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::or(inst_bin),
                        0b0000001 => Instruction::rem(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    0b111 => match Instruction::decode_funct7(inst_bin) {
                        0b0000000 => Instruction::and(inst_bin),
                        0b0000001 => Instruction::remu(inst_bin),
                        _ => Instruction::Illegal,
                    },
                    _ => Instruction::Illegal,
                }
                0b0001111 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => Instruction::fence(),
                    0b001 => Instruction::fence_i(),
                    _ => Instruction::Illegal,
                },
                0b1110011 => match Instruction::decode_funct3(inst_bin) {
                    0b000 => match (inst_bin >> 20) & 0xfff {
                        0b000000000000 => Instruction::ecall(),
                        0b000000000001 => Instruction::ebreak(),
                        0b001100000010 => Instruction::mret(),
                        0b000100000101 => Instruction::wfi(),
                        _ => Instruction::Illegal,
                    },
                    0b001 => Instruction::csrrw(inst_bin),
                    0b010 => Instruction::csrrs(inst_bin),
                    0b011 => Instruction::csrrc(inst_bin),
                    0b101 => Instruction::csrrwi(inst_bin),
                    0b110 => Instruction::csrrsi(inst_bin),
                    0b111 => Instruction::csrrci(inst_bin),
                    _ => Instruction::Illegal,
                },
                _ => Instruction::Illegal,
            }
            0b00 => match Instruction::decode_c_funct3(inst_bin as u16) {
                0b000 => Instruction::c_addi4spn(inst_bin as u16),
                0b010 => Instruction::c_lw(inst_bin as u16),
                0b110 => Instruction::c_sw(inst_bin as u16),
                _ => Instruction::Illegal,
            },
            0b01 => match Instruction::decode_c_funct3(inst_bin as u16) {
                0b000 => Instruction::c_addi(inst_bin as u16),
                0b001 => Instruction::c_jal(inst_bin as u16),
                0b010 => Instruction::c_li(inst_bin as u16),
                0b011 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    if rd == 2 {
                        Instruction::c_addi16sp(inst_bin as u16)
                    } else {
                        Instruction::c_lui(inst_bin as u16)
                    }
                }
                0b100 => match Instruction::decode_c_funct2(inst_bin as u16) {
                    0b00 => Instruction::c_srli(inst_bin as u16),
                    0b01 => Instruction::c_srai(inst_bin as u16),
                    0b10 => Instruction::c_andi(inst_bin as u16),
                    0b11 => match Instruction::decode_c_funct6(inst_bin as u16) {
                        0b100011 => match (inst_bin >> 5) & 0x3 {
                            0b00 => Instruction::c_sub(inst_bin as u16),
                            0b01 => Instruction::c_xor(inst_bin as u16),
                            0b10 => Instruction::c_or(inst_bin as u16),
                            0b11 => Instruction::c_and(inst_bin as u16),
                            _ => Instruction::Illegal,
                        },
                        _ => Instruction::Illegal,
                    }
                    _ => Instruction::Illegal,
                }
                0b101 => Instruction::c_j(inst_bin as u16),
                0b110 => Instruction::c_beqz(inst_bin as u16),
                0b111 => Instruction::c_bnez(inst_bin as u16),
                _ => Instruction::Illegal,
            },
            0b10 => match Instruction::decode_c_funct3(inst_bin as u16) {
                0b000 => Instruction::c_slli(inst_bin as u16),
                0b010 => Instruction::c_lwsp(inst_bin as u16),
                0b100 => {
                    let rs2 = (inst_bin >> 2) & 0x1f;
                    let rd = (inst_bin >> 7) & 0x1f;
                    match Instruction::decode_c_funct4(inst_bin as u16) {
                        0b1000 => {
                            if rd == 0 {
                                Instruction::Illegal
                            } else if rs2 == 0 {
                                Instruction::c_jr(inst_bin as u16)
                            } else {
                                Instruction::c_mv(inst_bin as u16)
                            }
                        }
                        0b1001 => {
                            if rd == 0 && rs2 == 0 {
                                Instruction::c_ebreak()
                            } else if rd == 0 {
                                Instruction::Illegal
                            } else if rs2 == 0 {
                                Instruction::c_jalr(inst_bin as u16)
                            } else {
                                Instruction::c_add(inst_bin as u16)
                            }
                        }
                        _ => Instruction::Illegal,
                    }
                }
                0b110 => Instruction::c_swsp(inst_bin as u16),
                _ => Instruction::Illegal,
            },
            _ => Instruction::Illegal,
        };
        (inst, if quadrant == 0b11 { 4 } else { 2 })
    }

    #[inline(always)]
    fn gen_inst<B: bus::Bus>(&mut self, raw_ptr: u32, bus: &mut B) -> (Instruction, u32) {
        let inst_low = bus.read16(raw_ptr);
        let quadrant = Instruction::decode_quadrant(inst_low);
        let (inst_bin, inst_size) = if quadrant == 0b11 {
            // 32-bit instruction
            let inst_high = bus.read16(raw_ptr + 2);
            (((inst_high as u32) << 16) | inst_low as u32, 4)
        } else {
            // 16-bit instruction
            (inst_low as u32, 2)
        };

        let (inst, _) = self.gen_inst_from_bin(inst_bin, quadrant);
        (inst, inst_size)
    }

    #[inline(always)]
    fn exec<B: bus::Bus>(&mut self, inst: Instruction, bus: &mut B) -> StepResult {
        // 命令実行の直前にキャッシュフラッシュが必要な場合がある（セルフモディファイングコード用）
        // ただし、通常のテストでは fence.i を明示的に呼ぶべき。
        // モックバス等での書き込みを即座に反映させるため、書き込み命令時にフラッシュする方針も検討。
        match inst {
            // U-Type
            Instruction::Lui   { rd, imm } => self.lui   (rd, imm),
            Instruction::Auipc { rd, imm } => self.auipc (rd, imm),

            // J-Type
            Instruction::Jal   { rd, imm } => self.jal (rd, imm),

            // I-Type
            Instruction::Jalr  { rd, rs1, imm } => self.jalr  (rd, rs1, imm),
            Instruction::Lb    { rd, rs1, imm } => self.lb    (rd, rs1, imm, bus),
            Instruction::Lh    { rd, rs1, imm } => self.lh    (rd, rs1, imm, bus),
            Instruction::Lw    { rd, rs1, imm } => self.lw    (rd, rs1, imm, bus),
            Instruction::Lbu   { rd, rs1, imm } => self.lbu   (rd, rs1, imm, bus),
            Instruction::Lhu   { rd, rs1, imm } => self.lhu   (rd, rs1, imm, bus),
            Instruction::Addi  { rd, rs1, imm } => self.addi  (rd, rs1, imm),
            Instruction::Slti  { rd, rs1, imm } => self.slti  (rd, rs1, imm),
            Instruction::Sltiu { rd, rs1, imm } => self.sltiu (rd, rs1, imm),
            Instruction::Xori  { rd, rs1, imm } => self.xori  (rd, rs1, imm),
            Instruction::Ori   { rd, rs1, imm } => self.ori   (rd, rs1, imm),
            Instruction::Andi  { rd, rs1, imm } => self.andi  (rd, rs1, imm),

            Instruction::Slli  { rd, rs1, shamt } => self.slli (rd, rs1, shamt),
            Instruction::Srli  { rd, rs1, shamt } => self.srli (rd, rs1, shamt),
            Instruction::Srai  { rd, rs1, shamt } => self.srai (rd, rs1, shamt),

            // B-Type
            Instruction::Beq   { rs1, rs2, imm } => self.beq  (rs1, rs2, imm),
            Instruction::Bne   { rs1, rs2, imm } => self.bne  (rs1, rs2, imm),
            Instruction::Blt   { rs1, rs2, imm } => self.blt  (rs1, rs2, imm),
            Instruction::Bge   { rs1, rs2, imm } => self.bge  (rs1, rs2, imm),
            Instruction::Bltu  { rs1, rs2, imm } => self.bltu (rs1, rs2, imm),
            Instruction::Bgeu  { rs1, rs2, imm } => self.bgeu (rs1, rs2, imm),

            // S-Type
            Instruction::Sb    { rs1, rs2, imm } => self.sb (rs1, rs2, imm, bus),
            Instruction::Sh    { rs1, rs2, imm } => self.sh (rs1, rs2, imm, bus),
            Instruction::Sw    { rs1, rs2, imm } => self.sw (rs1, rs2, imm, bus),

            // R-Type
            Instruction::Add    { rd, rs1, rs2 } => self.add    (rd, rs1, rs2),
            Instruction::Sub    { rd, rs1, rs2 } => self.sub    (rd, rs1, rs2),
            Instruction::Sll    { rd, rs1, rs2 } => self.sll    (rd, rs1, rs2),
            Instruction::Slt    { rd, rs1, rs2 } => self.slt    (rd, rs1, rs2),
            Instruction::Sltu   { rd, rs1, rs2 } => self.sltu   (rd, rs1, rs2),
            Instruction::Xor    { rd, rs1, rs2 } => self.xor    (rd, rs1, rs2),
            Instruction::Srl    { rd, rs1, rs2 } => self.srl    (rd, rs1, rs2),
            Instruction::Sra    { rd, rs1, rs2 } => self.sra    (rd, rs1, rs2),
            Instruction::Or     { rd, rs1, rs2 } => self.or     (rd, rs1, rs2),
            Instruction::And    { rd, rs1, rs2 } => self.and    (rd, rs1, rs2),

            Instruction::Mul    { rd, rs1, rs2 } => self.mul    (rd, rs1, rs2),
            Instruction::Mulh   { rd, rs1, rs2 } => self.mulh   (rd, rs1, rs2),
            Instruction::Mulhsu { rd, rs1, rs2 } => self.mulhsu (rd, rs1, rs2),
            Instruction::Mulhu  { rd, rs1, rs2 } => self.mulhu  (rd, rs1, rs2),
            Instruction::Div    { rd, rs1, rs2 } => self.div    (rd, rs1, rs2),
            Instruction::Divu   { rd, rs1, rs2 } => self.divu   (rd, rs1, rs2),
            Instruction::Rem    { rd, rs1, rs2 } => self.rem    (rd, rs1, rs2),
            Instruction::Remu   { rd, rs1, rs2 } => self.remu   (rd, rs1, rs2),

            Instruction::Fence  => self.fence(),
            Instruction::FenceI => self.fence_i(),

            Instruction::Ecall  => self.ecall(),
            Instruction::Ebreak => self.ebreak(),
            Instruction::Mret   => self.mret(),
            Instruction::Wfi    => StepResult::Ok(4),

            Instruction::Csrrw  { csr, rd, rs1 }  => self.csrrw  (csr, rd, rs1),
            Instruction::Csrrs  { csr, rd, rs1 }  => self.csrrs  (csr, rd, rs1),
            Instruction::Csrrc  { csr, rd, rs1 }  => self.csrrc  (csr, rd, rs1),
            Instruction::Csrrwi { csr, rd, uimm } => self.csrrwi (csr, rd, uimm),
            Instruction::Csrrsi { csr, rd, uimm } => self.csrrsi (csr, rd, uimm),
            Instruction::Csrrci { csr, rd, uimm } => self.csrrci (csr, rd, uimm),

            // C-Extension
            Instruction::CAddi4spn { rd, rs1, imm } => self.c_addi4spn(rd, rs1, imm),
            Instruction::CLw       { rd, rs1, imm } => self.c_lw(rd, rs1, imm, bus),
            Instruction::CSw       { rs1, rs2, imm } => self.c_sw(rs1, rs2, imm, bus),
            Instruction::CAddi     { rd, imm } => self.c_addi(rd, imm),
            Instruction::CJal      { rd, imm } => self.c_jal(rd, imm),
            Instruction::CLi       { rd, imm } => self.c_li(rd, imm),
            Instruction::CLui      { rd, imm } => self.c_lui(rd, imm),
            Instruction::CAddi16Sp { rd, imm } => self.c_addi16sp(rd, imm),
            Instruction::CSrli     { rd, shamt } => self.c_srli(rd, shamt),
            Instruction::Csrai     { rd, shamt } => self.c_srai(rd, shamt),
            Instruction::Candi     { rd, imm } => self.c_andi(rd, imm),
            Instruction::CSub      { rd, rs2 } => self.c_sub(rd, rs2),
            Instruction::CXor      { rd, rs2 } => self.c_xor(rd, rs2),
            Instruction::Cor       { rd, rs2 } => self.c_or(rd, rs2),
            Instruction::Cand      { rd, rs2 } => self.c_and(rd, rs2),
            Instruction::CJ        { imm } => self.c_j(imm),
            Instruction::CBeqz     { rs1, imm } => self.c_beqz(rs1, imm),
            Instruction::CBnez     { rs1, imm } => self.c_bnez(rs1, imm),
            Instruction::CSlli     { rd, shamt } => self.c_slli(rd, shamt),
            Instruction::CLwsp     { rd, imm } => self.c_lwsp(rd, imm, bus),
            Instruction::CJr       { rs1 }  => self.c_jr(rs1),
            Instruction::CMv       { rd, rs2 } => self.c_mv(rd, rs2),
            Instruction::CJalr     { rs1 } => self.c_jalr(rs1),
            Instruction::CAdd      { rd, rs2 } => self.c_add(rd, rs2),
            Instruction::CSwsp     { rs2, imm } => self.c_swsp(rs2, imm, bus),

            _ => StepResult::Trap(2),
        }
    }

    /// PC を指す命令がキャッシュされているページを無効化する
    pub fn flush_cache_line(&mut self, addr: u32) {
        let page_num = (addr / PAGE_SIZE as u32) as usize;
        if page_num < self.pages.len() {
            self.pages[page_num] = None;
        }
        // flush_cache_line が呼ばれたら、常に current_page_num をリセットし
        // fetch 時にページを再評価させる
        self.current_page_num = 0xffffffff;
    }

    /// 全てのページキャッシュを無効化する
    pub fn flush_all_cache(&mut self) {
        self.pages.iter_mut().for_each(|p| *p = None);
    }
}
