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
mod instruction;

use super::bus;
use csr::Csr;
use privilege_mode::PrivilegeMode;
use instruction::Instruction;

#[derive(Debug)]
pub enum StepResult {
    Ok,
    Trap(u32),
    Jumped,
}

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

    /// 命令キャッシュ (デコード済み命令を保持)
    cache: Vec<Option<Box<dyn Instruction>>>,
}

impl Cpu {
    pub fn new(pc: u32) -> Self {
        Self {
            regs: [0; 32],
            pc,
            csr: Csr::default(),
            mode: PrivilegeMode::Machine,
            cache: Vec::new(),
        }
    }

    /// 命令キャッシュをクリアする
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// 1ステップ実行
    pub fn step<B: bus::Bus>(&mut self, bus: &mut B) -> StepResult {
        // クロックを進める
        bus.tick();

        // 実行前に割り込みをチェック
        if let Some(interrupt_code) = self.check_interrupts(bus) {
            return self.handle_trap(interrupt_code, 0);
        }

        // キャッシュのサイズが足りない場合は拡張 (通常はプログラムロード時に設定するのが望ましいが)
        // ここでは簡易的に 1MB 程度の初期サイズを持たせるか、必要に応じて拡張する。
        let pc_idx = self.pc as usize;
        let (result, inst_bin) = if let Some(inst) = self.cache.get_mut(pc_idx).and_then(|o| o.take()) {
            // キャッシュヒット
            let bin = inst.get_inst_bin();
            let res = inst.execute(self, bus);
            // execute 内で cache.clear() (fence.i) が呼ばれた可能性があるため
            // 呼び出し後にキャッシュが存在するか確認して戻す
            if self.pc as usize == pc_idx && pc_idx < self.cache.len() {
                 self.cache[pc_idx] = Some(inst);
            }
            (res, bin)
        } else {
            // キャッシュミス: デコードしてキャッシュに格納
            let (inst_bin, quadrant) = self.fetch(bus);
            let inst_obj: Box<dyn Instruction> = if quadrant == 0b11 {
                self.decode32_to_obj(inst_bin)
            } else {
                self.decode16_to_obj(inst_bin as u16, quadrant)
            };
            let res = inst_obj.execute(self, bus);
            let bin = inst_obj.get_inst_bin();
            // 同様に execute 内で cache.clear() された可能性を考慮
            if self.pc as usize == pc_idx {
                if pc_idx >= self.cache.len() {
                    self.cache.resize_with(pc_idx + 0x1000, || None);
                }
                self.cache[pc_idx] = Some(inst_obj);
            }
            (res, bin)
        };

        // 実行結果に基づく PC の更新
        let pc_inc = if (inst_bin & 0x3) == 0b11 { 4 } else { 2 };

        match result {
            StepResult::Ok => {
                self.pc += pc_inc;
                StepResult::Ok
            }
            StepResult::Trap(code) => {
                let mtval = if code == 2 {
                    inst_bin
                } else {
                    0
                };
                self.handle_trap(code, mtval);
                StepResult::Trap(code)
            }
            _ => result,
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

    fn fetch<B: bus::Bus>(&mut self, bus: &mut B) -> (u32, u16) {
        let inst_low = bus.read16(self.pc);
        let quadrant = self.decode_quadrant(inst_low);
        if quadrant == 0b11 {
            // 32-bit instruction
            let inst_high = bus.read16(self.pc + 2);
            let inst_bin = ((inst_high as u32) << 16) | inst_low as u32;
            (inst_bin, quadrant)
        } else {
            // 16-bit instruction
            (inst_low as u32, quadrant)
        }
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

    fn decode32_to_obj(&self, inst_bin: u32) -> Box<dyn Instruction> {
        let opcode = self.decode_opcode(inst_bin);
        let funct3 = self.decode_funct3(inst_bin);

        match opcode {
            0b0110111 => {
                let (rd, imm) = self.decode_u_type(inst_bin);
                Box::new(instruction::UType { inst_bin, rd, imm, executor: Cpu::lui })
            }
            0b0010111 => {
                let (rd, imm) = self.decode_u_type(inst_bin);
                Box::new(instruction::UType { inst_bin, rd, imm, executor: Cpu::auipc })
            }
            0b1101111 => {
                let (rd, imm) = self.decode_j_type(inst_bin);
                Box::new(instruction::JType { inst_bin, rd, imm, executor: Cpu::jal })
            }
            0b1100111 => {
                let (rd, rs1, imm) = self.decode_i_type(inst_bin);
                Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::jalr })
            }
            0b0010011 => {
                let (rd, rs1, imm) = self.decode_i_type(inst_bin);
                match funct3 {
                    0b000 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::addi }),
                    0b001 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::slli }),
                    0b010 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::slti }),
                    0b011 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::sltiu }),
                    0b100 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::xori }),
                    0b101 => {
                        let funct7 = self.decode_funct7(inst_bin);
                        match funct7 {
                            0b0000000 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::srli }),
                            0b0100000 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::srai }),
                            _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                        }
                    }
                    0b110 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::ori }),
                    0b111 => Box::new(instruction::IType { inst_bin, rd, rs1, imm, executor: Cpu::andi }),
                    _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                }
            }
            0b1100011 => {
                let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
                match funct3 {
                    0b000 => Box::new(instruction::BType { inst_bin, rs1, rs2, imm, executor: Cpu::beq }),
                    0b001 => Box::new(instruction::BType { inst_bin, rs1, rs2, imm, executor: Cpu::bne }),
                    0b100 => Box::new(instruction::BType { inst_bin, rs1, rs2, imm, executor: Cpu::blt }),
                    0b101 => Box::new(instruction::BType { inst_bin, rs1, rs2, imm, executor: Cpu::bge }),
                    0b110 => Box::new(instruction::BType { inst_bin, rs1, rs2, imm, executor: Cpu::bltu }),
                    0b111 => Box::new(instruction::BType { inst_bin, rs1, rs2, imm, executor: Cpu::bgeu }),
                    _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                }
            }
            0b0000011 => {
                let (rd, rs1, imm) = self.decode_i_type(inst_bin);
                match funct3 {
                    0b000 => Box::new(instruction::ITypeLoad { inst_bin, rd, rs1, imm, executor: Cpu::lb }),
                    0b001 => Box::new(instruction::ITypeLoad { inst_bin, rd, rs1, imm, executor: Cpu::lh }),
                    0b010 => Box::new(instruction::ITypeLoad { inst_bin, rd, rs1, imm, executor: Cpu::lw }),
                    0b100 => Box::new(instruction::ITypeLoad { inst_bin, rd, rs1, imm, executor: Cpu::lbu }),
                    0b101 => Box::new(instruction::ITypeLoad { inst_bin, rd, rs1, imm, executor: Cpu::lhu }),
                    _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                }
            }
            0b0100011 => {
                let (rs1, rs2, imm) = self.decode_s_type(inst_bin);
                match funct3 {
                    0b000 => Box::new(instruction::SType { inst_bin, rs1, rs2, imm, executor: Cpu::sb }),
                    0b001 => Box::new(instruction::SType { inst_bin, rs1, rs2, imm, executor: Cpu::sh }),
                    0b010 => Box::new(instruction::SType { inst_bin, rs1, rs2, imm, executor: Cpu::sw }),
                    _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                }
            }
            0b0110011 => {
                let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
                let funct7 = self.decode_funct7(inst_bin);
                match funct3 {
                    0b000 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::add }),
                        0b0100000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::sub }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::mul }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b001 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::sll }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::mulh }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b010 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::slt }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::mulhsu }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b011 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::sltu }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::mulhu }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b100 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::xor }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::div }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b101 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::srl }),
                        0b0100000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::sra }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::divu }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b110 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::or }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::rem }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    0b111 => match funct7 {
                        0b0000000 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::and }),
                        0b0000001 => Box::new(instruction::RType { inst_bin, rd, rs1, rs2, executor: Cpu::remu }),
                        _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                    },
                    _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                }
            }
            0b0001111 => match funct3 {
                0b000 => Box::new(instruction::NoArgsType { inst_bin, executor: Cpu::fence }),
                0b001 => Box::new(instruction::NoArgsType { inst_bin, executor: Cpu::fence_i }),
                _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
            },
            0b1110011 => match funct3 {
                0b000 => match (inst_bin >> 20) & 0xfff {
                    0b000000000000 => Box::new(instruction::NoArgsType { inst_bin, executor: Cpu::ecall }),
                    0b000000000001 => Box::new(instruction::NoArgsType { inst_bin, executor: Cpu::ebreak }),
                    0b001100000010 => Box::new(instruction::NoArgsType { inst_bin, executor: Cpu::mret }),
                    0b000100000101 => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Ok }), // wfi: NOP
                    _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
                },
                0b001 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
                    let csr_addr = (inst_bin >> 20) & 0xfff;
                    Box::new(instruction::CSRType { inst_bin, rd, rs1_uimm: rs1, csr_addr, executor: Cpu::csrrw })
                }
                0b010 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
                    let csr_addr = (inst_bin >> 20) & 0xfff;
                    Box::new(instruction::CSRType { inst_bin, rd, rs1_uimm: rs1, csr_addr, executor: Cpu::csrrs })
                }
                0b011 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
                    let csr_addr = (inst_bin >> 20) & 0xfff;
                    Box::new(instruction::CSRType { inst_bin, rd, rs1_uimm: rs1, csr_addr, executor: Cpu::csrrc })
                }
                0b101 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    let uimm = (inst_bin >> 15) & 0x1f;
                    let csr_addr = (inst_bin >> 20) & 0xfff;
                    Box::new(instruction::CSRType { inst_bin, rd, rs1_uimm: uimm as usize, csr_addr, executor: Cpu::csrrwi })
                }
                0b110 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    let uimm = (inst_bin >> 15) & 0x1f;
                    let csr_addr = (inst_bin >> 20) & 0xfff;
                    Box::new(instruction::CSRType { inst_bin, rd, rs1_uimm: uimm as usize, csr_addr, executor: Cpu::csrrsi })
                }
                0b111 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    let uimm = (inst_bin >> 15) & 0x1f;
                    let csr_addr = (inst_bin >> 20) & 0xfff;
                    Box::new(instruction::CSRType { inst_bin, rd, rs1_uimm: uimm as usize, csr_addr, executor: Cpu::csrrci })
                }
                _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
            },
            _ => Box::new(instruction::NoArgsType { inst_bin, executor: |_| StepResult::Trap(2) }),
        }
    }

    fn decode16_to_obj(&self, inst_bin: u16, quadrant: u16) -> Box<dyn Instruction> {
        match quadrant {
            0b00 => match self.decode_c_funct3(inst_bin) {
                0b000 => {
                    let (rd, imm) = self.decode_ciw_type(inst_bin);
                    Box::new(instruction::CType { inst_bin, rd, imm, executor: Cpu::c_addi4spn })
                }
                0b010 => {
                    let (rd, rs1, imm) = self.decode_cl_type(inst_bin);
                    Box::new(instruction::CTypeThreeBus { inst_bin, r1: rd, r2: rs1, imm, executor: |cpu, rd, rs1, imm, bus| cpu.c_lw(rd, rs1, imm, bus) })
                }
                0b110 => {
                    let (rs1, rs2, imm) = self.decode_cs_type(inst_bin);
                    Box::new(instruction::CTypeThreeBus { inst_bin, r1: rs1, r2: rs2, imm, executor: |cpu, rs1, rs2, imm, bus| cpu.c_sw(rs1, rs2, imm, bus) })
                }
                _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
            },
            0b01 => match self.decode_c_funct3(inst_bin) {
                0b000 => {
                    let (rd, imm) = self.decode_ci_type(inst_bin);
                    if rd == 0 {
                        // C.ADDI rd=0 is HINT
                        Box::new(instruction::CType { inst_bin, rd, imm, executor: Cpu::c_addi })
                    } else {
                        Box::new(instruction::CType { inst_bin, rd, imm, executor: Cpu::c_addi })
                    }
                }
                0b001 => {
                    let imm = self.decode_cj_type(inst_bin);
                    Box::new(instruction::CType { inst_bin, rd: 0, imm, executor: |cpu, _, imm| cpu.c_jal(imm) })
                }
                0b010 => {
                    let (rd, imm) = self.decode_ci_type(inst_bin);
                    if rd == 0 {
                        Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) })
                    } else {
                        Box::new(instruction::CType { inst_bin, rd, imm, executor: Cpu::c_li })
                    }
                }
                0b011 => {
                    let rd = ((inst_bin >> 7) & 0x1f) as usize;
                    if rd == 2 {
                        let imm = self.decode_c_addi16sp_imm(inst_bin);
                        Box::new(instruction::CType { inst_bin, rd: 0, imm, executor: |cpu, _, imm| cpu.c_addi16sp(imm) })
                    } else if rd == 0 {
                        Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) })
                    } else {
                        let (rd, imm) = self.decode_ci_type(inst_bin);
                        Box::new(instruction::CType { inst_bin, rd, imm, executor: Cpu::c_lui })
                    }
                }
                0b100 => match self.decode_c_funct2(inst_bin) {
                    0b00 => {
                        let (rd, shamt) = self.decode_cb_shamt_type(inst_bin);
                        if (inst_bin >> 12) & 0x1 != 0 {
                            Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) })
                        } else {
                            Box::new(instruction::CType { inst_bin, rd, imm: shamt & 0x1f, executor: Cpu::c_srli })
                        }
                    }
                    0b01 => {
                        let (rd, shamt) = self.decode_cb_shamt_type(inst_bin);
                        if (inst_bin >> 12) & 0x1 != 0 {
                            Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) })
                        } else {
                            Box::new(instruction::CType { inst_bin, rd, imm: shamt & 0x1f, executor: Cpu::c_srai })
                        }
                    }
                    0b10 => {
                        let (rd, imm) = self.decode_cb_andi_type(inst_bin);
                        Box::new(instruction::CType { inst_bin, rd, imm, executor: Cpu::c_andi })
                    }
                    0b11 => match self.decode_c_funct6(inst_bin) {
                        0b100011 => {
                             let (rd, rs2) = self.decode_ca_type(inst_bin);
                             match (inst_bin >> 5) & 0x3 {
                                0b00 => Box::new(instruction::CTypeTwo { inst_bin, r1: rd, r2: rs2, executor: Cpu::c_sub }),
                                0b01 => Box::new(instruction::CTypeTwo { inst_bin, r1: rd, r2: rs2, executor: Cpu::c_xor }),
                                0b10 => Box::new(instruction::CTypeTwo { inst_bin, r1: rd, r2: rs2, executor: Cpu::c_or }),
                                0b11 => Box::new(instruction::CTypeTwo { inst_bin, r1: rd, r2: rs2, executor: Cpu::c_and }),
                                _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
                            }
                        }
                        _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
                    }
                    _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
                }
                0b101 => {
                    let imm = self.decode_cj_type(inst_bin);
                    Box::new(instruction::CType { inst_bin, rd: 0, imm, executor: |cpu, _, imm| cpu.c_j(imm) })
                }
                0b110 => {
                    let (rs1, imm) = self.decode_cb_branch_type(inst_bin);
                    Box::new(instruction::CTypeThree { inst_bin, r1: rs1, r2: 0, imm, executor: |cpu, rs1, _, imm| cpu.c_beqz(rs1, imm) })
                }
                0b111 => {
                    let (rs1, imm) = self.decode_cb_branch_type(inst_bin);
                    Box::new(instruction::CTypeThree { inst_bin, r1: rs1, r2: 0, imm, executor: |cpu, rs1, _, imm| cpu.c_bnez(rs1, imm) })
                }
                _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
            },
            0b10 => match self.decode_c_funct3(inst_bin) {
                0b000 => {
                    let (rd, shamt) = self.decode_ci_shamt_type(inst_bin);
                    if rd == 0 {
                        Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) })
                    } else if (inst_bin >> 12) & 0x1 != 0 {
                        Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) })
                    } else {
                        Box::new(instruction::CType { inst_bin, rd, imm: shamt & 0x1f, executor: Cpu::c_slli })
                    }
                }
                0b010 => {
                    let (rd, imm) = self.decode_c_lwsp_type(inst_bin);
                    Box::new(instruction::CTypeLoad { inst_bin, rd, imm, executor: Cpu::c_lwsp })
                }
                0b100 => {
                    let rs2 = ((inst_bin >> 2) & 0x1f) as usize;
                    match self.decode_c_funct4(inst_bin) {
                        0b1000 => {
                            if rs2 == 0 {
                                let rs1 = ((inst_bin >> 7) & 0x1f) as usize;
                                Box::new(instruction::CTypeReg { inst_bin, reg: rs1, executor: Cpu::c_jr })
                            } else {
                                let rd = ((inst_bin >> 7) & 0x1f) as usize;
                                Box::new(instruction::CTypeTwo { inst_bin, r1: rd, r2: rs2, executor: Cpu::c_mv })
                            }
                        }
                        0b1001 => {
                            let rd = ((inst_bin >> 7) & 0x1f) as usize;
                            if rd == 0 && rs2 == 0 {
                                Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: Cpu::ebreak })
                            } else if rs2 == 0 {
                                Box::new(instruction::CTypeReg { inst_bin, reg: rd, executor: Cpu::c_jalr })
                            } else {
                                Box::new(instruction::CTypeTwo { inst_bin, r1: rd, r2: rs2, executor: Cpu::c_add })
                            }
                        }
                        _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
                    }
                }
                0b110 => {
                    let (rs2, imm) = self.decode_c_swsp_type(inst_bin);
                    Box::new(instruction::CTypeThreeBus { inst_bin, r1: rs2, r2: 0, imm, executor: |cpu, rs2, _, imm, bus| cpu.c_swsp(rs2, imm, bus) })
                }
                _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
            },
            _ => Box::new(instruction::NoArgsType { inst_bin: inst_bin as u32, executor: |_| StepResult::Trap(2) }),
        }
    }
}
