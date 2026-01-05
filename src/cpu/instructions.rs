#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum Instruction {
    None,    // キャッシュされていない命令であることを表す.
    Illegal, // 違法命令.

    Lui   { rd: u8, imm: u32 },
    Auipc { rd: u8, imm: u32 },

    Jal { rd: u8, imm: u32 },

    Jalr { rd: u8, rs1: u8, imm: u16 },

    Beq  { rs1: u8, rs2: u8, imm: u16 },
    Bne  { rs1: u8, rs2: u8, imm: u16 },
    Blt  { rs1: u8, rs2: u8, imm: u16 },
    Bge  { rs1: u8, rs2: u8, imm: u16 },
    Bltu { rs1: u8, rs2: u8, imm: u16 },
    Bgeu { rs1: u8, rs2: u8, imm: u16 },

    Lb  { rd: u8, rs1: u8, imm: u16 },
    Lh  { rd: u8, rs1: u8, imm: u16 },
    Lw  { rd: u8, rs1: u8, imm: u16 },
    Lbu { rd: u8, rs1: u8, imm: u16 },
    Lhu { rd: u8, rs1: u8, imm: u16 },

    Sb { rs2: u8, rs1: u8, imm: u16 },
    Sh { rs2: u8, rs1: u8, imm: u16 },
    Sw { rs2: u8, rs1: u8, imm: u16 },

    Addi  { rd: u8, rs1: u8, imm: u16 },
    Slti  { rd: u8, rs1: u8, imm: u16 },
    Sltiu { rd: u8, rs1: u8, imm: u16 },
    Xori  { rd: u8, rs1: u8, imm: u16 },
    Ori   { rd: u8, rs1: u8, imm: u16 },
    Andi  { rd: u8, rs1: u8, imm: u16 },

    Slli  { rd: u8, rs1: u8, shamt: u8 },
    Srli  { rd: u8, rs1: u8, shamt: u8 },
    Srai  { rd: u8, rs1: u8, shamt: u8 },

    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    Sll { rd: u8, rs1: u8, rs2: u8 },
    Slt { rd: u8, rs1: u8, rs2: u8 },
    Sltu { rd: u8, rs1: u8, rs2: u8 },
    Xor { rd: u8, rs1: u8, rs2: u8 },
    Srl { rd: u8, rs1: u8, rs2: u8 },
    Sra { rd: u8, rs1: u8, rs2: u8 },
    Or  { rd: u8, rs1: u8, rs2: u8 },
    And { rd: u8, rs1: u8, rs2: u8 },

    Fence,
    FenceI,

    Wfi,

    Ecall,
    Ebreak,
    Mret,

    Mul    { rd: u8, rs1: u8, rs2: u8 },
    Mulh   { rd: u8, rs1: u8, rs2: u8 },
    Mulhsu { rd: u8, rs1: u8, rs2: u8 },
    Mulhu  { rd: u8, rs1: u8, rs2: u8 },
    Div    { rd: u8, rs1: u8, rs2: u8 },
    Divu   { rd: u8, rs1: u8, rs2: u8 },
    Rem    { rd: u8, rs1: u8, rs2: u8 },
    Remu   { rd: u8, rs1: u8, rs2: u8 },

    CAddi4spn { rd: u8, rs1: u8, imm: u16 },
    CLw       { rd: u8, rs1: u8, imm: u16 },
    CSw       { rs2: u8, rs1: u8, imm: u16 },
    CAddi     { rd: u8, imm: i16 },
    CJal      { rd: u8, imm: i16 },
    CLi       { rd: u8, imm: i16 },
    CLui      { rd: u8, imm: i32 },
    CAddi16Sp { rd: u8, imm: i16 },
    CSrli     { rd: u8, shamt: u8},
    Csrai     { rd: u8, shamt: u8},
    Candi     { rd: u8, imm: i16},
    CSub      { rd: u8, rs2: u8},
    CXor      { rd: u8, rs2: u8},
    Cor       { rd: u8, rs2: u8},
    Cand      { rd: u8, rs2: u8},
    CJ        { imm: i16 },
    CBeqz     { rs1: u8, imm: i16 },
    CBnez     { rs1: u8, imm: i16 },
    CSlli     { rd: u8, shamt: u8},
    CLwsp     { rd: u8, imm: u16},
    CJr       { rs1: u8 },
    CMv       { rd: u8, rs2: u8 },
    CJalr     { rs1: u8 },
    CAdd      { rd: u8, rs2: u8 },
    CSwsp     { rs2: u8, imm: u16 },

    Csrrw  { csr: u16, rd: u8, rs1: u8 },
    Csrrs  { csr: u16, rd: u8, rs1: u8 },
    Csrrc  { csr: u16, rd: u8, rs1: u8 },
    Csrrwi { csr: u16, rd: u8, uimm: u8 },
    Csrrsi { csr: u16, rd: u8, uimm: u8 },
    Csrrci { csr: u16, rd: u8, uimm: u8 },
}

impl Instruction {
    #[inline(always)]
        pub fn lui(inst_bin: u32) -> Instruction {
        let (rd, imm) = Instruction::decode_u_type(inst_bin);
        Instruction::Lui { rd, imm }
    }

    #[inline(always)]
    pub fn auipc(inst_bin: u32) -> Instruction {
        let (rd, imm) = Instruction::decode_u_type(inst_bin);
        Instruction::Auipc { rd, imm }
    }

    #[inline(always)]
    pub fn jal(inst_bin: u32) -> Instruction {
        let (rd, imm) = Instruction::decode_j_type(inst_bin);
        Instruction::Jal { rd, imm }
    }

    #[inline(always)]
    pub fn jalr(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Jalr { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn beq(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_b_type(inst_bin);
        Instruction::Beq { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn bne(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_b_type(inst_bin);
        Instruction::Bne { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn blt(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_b_type(inst_bin);
        Instruction::Blt { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn bge(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_b_type(inst_bin);
        Instruction::Bge { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn bltu(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_b_type(inst_bin);
        Instruction::Bltu { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn bgeu(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_b_type(inst_bin);
        Instruction::Bgeu { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn lb(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Lb { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn lh(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Lh { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn lw(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Lw { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn lbu(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Lbu { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn lhu(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Lhu { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn sb(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_s_type(inst_bin);
        Instruction::Sb { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn sh(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_s_type(inst_bin);
        Instruction::Sh { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn sw(inst_bin: u32) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_s_type(inst_bin);
        Instruction::Sw { rs1, rs2, imm }
    }

    #[inline(always)]
    pub fn addi(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Addi { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn slti(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Slti { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn sltiu(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Sltiu { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn xori(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Xori { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn ori(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Ori { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn andi(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        Instruction::Andi { rd, rs1, imm }
    }

    #[inline(always)]
    pub fn slli(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        if (imm & 0xfe0) != 0 {
            return Instruction::Illegal;
        }
        let shamt = (imm & 0x1f) as u8;
        Instruction::Slli { rd, rs1, shamt }
    }

    #[inline(always)]
    pub fn srli(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        if (imm & 0xfe0) != 0 {
            return Instruction::Illegal;
        }
        let shamt = (imm & 0x1f) as u8;
        Instruction::Srli { rd, rs1, shamt }
    }

    #[inline(always)]
    pub fn srai(inst_bin: u32) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_i_type(inst_bin);
        if (imm & 0xbe0) != 0 {
            return Instruction::Illegal;
        }
        let shamt = (imm & 0x1f) as u8;
        Instruction::Srai { rd, rs1, shamt }
    }

    #[inline(always)]
    pub fn add(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Add { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn sub(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Sub { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn sll(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Sll { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn slt(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Slt { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn sltu(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Sltu { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn xor(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Xor { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn srl(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Srl { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn sra(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Sra { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn or(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Or { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn and(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::And { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn fence() -> Instruction {
        Instruction::Fence
    }

    #[inline(always)]
    pub fn fence_i() -> Instruction {
        Instruction::FenceI
    }

    #[inline(always)]
    pub fn ecall() -> Instruction {
        Instruction::Ecall
    }

    #[inline(always)]
    pub fn ebreak() -> Instruction {
        Instruction::Ebreak
    }

    #[inline(always)]
    pub fn mret() -> Instruction {
        Instruction::Mret
    }

    #[inline(always)]
    pub fn wfi() -> Instruction {
        Instruction::Wfi
    }

    #[inline(always)]
    pub fn mul(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Mul { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn mulh(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Mulh { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn mulhsu(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Mulhsu { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn mulhu(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Mulhu { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn div(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Div { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn divu(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Divu { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn rem(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Rem { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn remu(inst_bin: u32) -> Instruction {
        let (rd, rs1, rs2) = Instruction::decode_r_type(inst_bin);
        Instruction::Remu { rd, rs1, rs2 }
    }

    #[inline(always)]
    pub fn csrrw(inst_bin: u32) -> Instruction {
        let (csr, rd, rs1) = Instruction::decode_csr_type(inst_bin);
        Instruction::Csrrw { csr, rd, rs1 }
    }

    #[inline(always)]
    pub fn csrrs(inst_bin: u32) -> Instruction {
        let (csr, rd, rs1) = Instruction::decode_csr_type(inst_bin);
        Instruction::Csrrs { csr, rd, rs1 }
    }

    #[inline(always)]
    pub fn csrrc(inst_bin: u32) -> Instruction {
        let (csr, rd, rs1) = Instruction::decode_csr_type(inst_bin);
        Instruction::Csrrc { csr, rd, rs1 }
    }

    #[inline(always)]
    pub fn csrrwi(inst_bin: u32) -> Instruction {
        let (csr, rd, uimm) = Instruction::decode_csr_i_type(inst_bin);
        Instruction::Csrrwi { csr, rd, uimm }
    }

    #[inline(always)]
    pub fn csrrsi(inst_bin: u32) -> Instruction {
        let (csr, rd, uimm) = Instruction::decode_csr_i_type(inst_bin);
        Instruction::Csrrsi { csr, rd, uimm }
    }

    #[inline(always)]
    pub fn csrrci(inst_bin: u32) -> Instruction {
        let (csr, rd, uimm) = Instruction::decode_csr_i_type(inst_bin);
        Instruction::Csrrci { csr, rd, uimm }
    }

    // C-Extension
    #[inline(always)]
    pub fn c_addi4spn(inst_bin: u16) -> Instruction {
        let (rd, imm) = Instruction::decode_ciw_type(inst_bin);
        Instruction::CAddi4spn { rd: rd as u8, rs1: 2, imm: imm as u16 }
    }
    #[inline(always)]
    pub fn c_lw(inst_bin: u16) -> Instruction {
        let (rd, rs1, imm) = Instruction::decode_cl_type(inst_bin);
        Instruction::CLw { rd: rd as u8, rs1: rs1 as u8, imm: imm as u16 }
    }
    #[inline(always)]
    pub fn c_sw(inst_bin: u16) -> Instruction {
        let (rs1, rs2, imm) = Instruction::decode_cs_type(inst_bin);
        Instruction::CSw { rs1: rs1 as u8, rs2: rs2 as u8, imm: imm as u16 }
    }
    #[inline(always)]
    pub fn c_addi(inst_bin: u16) -> Instruction {
        let (rd, imm) = Instruction::decode_ci_type(inst_bin);
        Instruction::CAddi { rd: rd as u8, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_jal(inst_bin: u16) -> Instruction {
        let imm = Instruction::decode_cj_type(inst_bin);
        Instruction::CJal { rd: 1, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_li(inst_bin: u16) -> Instruction {
        let (rd, imm) = Instruction::decode_ci_type(inst_bin);
        Instruction::CLi { rd: rd as u8, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_addi16sp(inst_bin: u16) -> Instruction {
        let imm = Instruction::decode_c_addi16sp_imm(inst_bin);
        Instruction::CAddi16Sp { rd: 2, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_lui(inst_bin: u16) -> Instruction {
        let (rd, imm) = Instruction::decode_ci_type(inst_bin);
        Instruction::CLui { rd: rd as u8, imm: imm as i32 }
    }
    #[inline(always)]
    pub fn c_srli(inst_bin: u16) -> Instruction {
        let (rd, shamt) = Instruction::decode_cb_shamt_type(inst_bin);
        if (inst_bin >> 12) & 0x1 != 0 {
            return Instruction::Illegal;
        }
        Instruction::CSrli { rd: rd as u8, shamt: shamt as u8 }
    }
    #[inline(always)]
    pub fn c_srai(inst_bin: u16) -> Instruction {
        let (rd, shamt) = Instruction::decode_cb_shamt_type(inst_bin);
        if (inst_bin >> 12) & 0x1 != 0 {
            return Instruction::Illegal;
        }
        Instruction::Csrai { rd: rd as u8, shamt: shamt as u8 }
    }
    #[inline(always)]
    pub fn c_andi(inst_bin: u16) -> Instruction {
        let (rd, imm) = Instruction::decode_cb_andi_type(inst_bin);
        Instruction::Candi { rd: rd as u8, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_sub(inst_bin: u16) -> Instruction {
        let (rd, rs2) = Instruction::decode_ca_type(inst_bin);
        Instruction::CSub { rd: rd as u8, rs2: rs2 as u8 }
    }
    #[inline(always)]
    pub fn c_xor(inst_bin: u16) -> Instruction {
        let (rd, rs2) = Instruction::decode_ca_type(inst_bin);
        Instruction::CXor { rd: rd as u8, rs2: rs2 as u8 }
    }
    #[inline(always)]
    pub fn c_or(inst_bin: u16) -> Instruction {
        let (rd, rs2) = Instruction::decode_ca_type(inst_bin);
        Instruction::Cor { rd: rd as u8, rs2: rs2 as u8 }
    }
    #[inline(always)]
    pub fn c_and(inst_bin: u16) -> Instruction {
        let (rd, rs2) = Instruction::decode_ca_type(inst_bin);
        Instruction::Cand { rd: rd as u8, rs2: rs2 as u8 }
    }
    #[inline(always)]
    pub fn c_j(inst_bin: u16) -> Instruction {
        let imm = Instruction::decode_cj_type(inst_bin);
        Instruction::CJ { imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_beqz(inst_bin: u16) -> Instruction {
        let (rs1, imm) = Instruction::decode_cb_branch_type(inst_bin);
        Instruction::CBeqz { rs1: rs1 as u8, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_bnez(inst_bin: u16) -> Instruction {
        let (rs1, imm) = Instruction::decode_cb_branch_type(inst_bin);
        Instruction::CBnez { rs1: rs1 as u8, imm: imm as i16 }
    }
    #[inline(always)]
    pub fn c_slli(inst_bin: u16) -> Instruction {
        let (rd, shamt) = Instruction::decode_ci_shamt_type(inst_bin);
        if (inst_bin >> 12) & 0x1 != 0 {
            return Instruction::Illegal;
        }
        Instruction::CSlli { rd: rd as u8, shamt: shamt as u8 }
    }
    #[inline(always)]
    pub fn c_lwsp(inst_bin: u16) -> Instruction {
        let (rd, imm) = Instruction::decode_c_lwsp_type(inst_bin);
        Instruction::CLwsp { rd: rd as u8, imm: imm as u16 }
    }
    #[inline(always)]
    pub fn c_jr(inst_bin: u16) -> Instruction {
        let (rs1, _) = Instruction::decode_cr_type(inst_bin);
        Instruction::CJr { rs1: rs1 as u8 }
    }
    #[inline(always)]
    pub fn c_mv(inst_bin: u16) -> Instruction {
        let (rd, rs2) = Instruction::decode_cr_type(inst_bin);
        Instruction::CMv { rd: rd as u8, rs2: rs2 as u8 }
    }
    #[inline(always)]
    pub fn c_ebreak() -> Instruction {
        Instruction::Ebreak
    }
    #[inline(always)]
    pub fn c_jalr(inst_bin: u16) -> Instruction {
        let (rs1, _) = Instruction::decode_cr_type(inst_bin);
        Instruction::CJalr { rs1: rs1 as u8 }
    }
    #[inline(always)]
    pub fn c_add(inst_bin: u16) -> Instruction {
        let (rd, rs2) = Instruction::decode_cr_type(inst_bin);
        Instruction::CAdd { rd: rd as u8, rs2: rs2 as u8 }
    }
    #[inline(always)]
    pub fn c_swsp(inst_bin: u16) -> Instruction {
        let (rs2, imm) = Instruction::decode_c_swsp_type(inst_bin);
        Instruction::CSwsp { rs2: rs2 as u8, imm: imm as u16 }
    }
}
