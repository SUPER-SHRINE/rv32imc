use super::Cpu;

impl Cpu {
    pub(super) fn decode_i_type(&self, inst_bin: u32) -> (usize, usize, u32) {
        let rd = ((inst_bin >> 7) & 0x1f) as usize;
        let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
        let imm = (inst_bin as i32 >> 20) as u32; // Sign extension
        (rd, rs1, imm)
    }

    pub(super) fn decode_u_type(&self, inst_bin: u32) -> (usize, u32) {
        let rd = ((inst_bin >> 7) & 0x1f) as usize;
        let imm = inst_bin & 0xffff_f000;
        (rd, imm)
    }

    pub(super) fn decode_j_type(&self, inst_bin: u32) -> (usize, u32) {
        let rd = ((inst_bin >> 7) & 0x1f) as usize;
        let imm20 = (inst_bin >> 31) & 0x1;
        let imm10_1 = (inst_bin >> 21) & 0x3ff;
        let imm11 = (inst_bin >> 20) & 0x1;
        let imm19_12 = (inst_bin >> 12) & 0xff;

        let imm = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);

        // Sign extension from 21st bit
        let imm = if imm20 != 0 {
            imm | 0xffe0_0000
        } else {
            imm
        };

        (rd, imm)
    }

    pub(super) fn decode_b_type(&self, inst_bin: u32) -> (usize, usize, u32) {
        let imm12 = (inst_bin >> 31) & 0x1;
        let imm10_5 = (inst_bin >> 25) & 0x3f;
        let rs2 = ((inst_bin >> 20) & 0x1f) as usize;
        let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
        let imm4_1 = (inst_bin >> 8) & 0xf;
        let imm11 = (inst_bin >> 7) & 0x1;

        let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);

        // Sign extension from 13th bit
        let imm = if imm12 != 0 {
            imm | 0xffffe000
        } else {
            imm
        };

        (rs1, rs2, imm)
    }

    pub(super) fn decode_s_type(&self, inst_bin: u32) -> (usize, usize, u32) {
        let imm11_5 = (inst_bin >> 25) & 0x7f;
        let rs2 = ((inst_bin >> 20) & 0x1f) as usize;
        let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
        let imm4_0 = (inst_bin >> 7) & 0x1f;

        let imm = (imm11_5 << 5) | imm4_0;

        // Sign extension from 12th bit
        let imm = ((imm as i32) << 20 >> 20) as u32;

        (rs1, rs2, imm)
    }

    pub(super) fn decode_r_type(&self, inst_bin: u32) -> (usize, usize, usize) {
        let rs2 = ((inst_bin >> 20) & 0x1f) as usize;
        let rs1 = ((inst_bin >> 15) & 0x1f) as usize;
        let rd = ((inst_bin >> 7) & 0x1f) as usize;
        (rd, rs1, rs2)
    }

    pub(super) fn decode_ciw_type(&self, inst_bin: u16) -> (usize, u32) {
        let rd_prime = ((inst_bin >> 2) & 0x7) as usize;

        // nzuimm[9:2] bit structure in C.ADDI4SPN:
        // inst[12:5] is nzuimm[5:4|9:6|2|3]
        // bits in inst_bin (16-bit):
        // 12 11 | 10 9 8 7 | 6 | 5
        // imm[5:4] | imm[9:6] | imm[2] | imm[3]

        let i5_4 = (inst_bin >> 11) & 0x3;
        let i9_6 = (inst_bin >> 7) & 0xf;
        let i2 = (inst_bin >> 6) & 0x1;
        let i3 = (inst_bin >> 5) & 0x1;

        let nzuimm = (i5_4 << 4) | (i9_6 << 6) | (i2 << 2) | (i3 << 3);

        (8 + rd_prime, nzuimm as u32)
    }

    pub(super) fn decode_opcode(&self, inst_bin: u32) -> u32 {
        inst_bin & 0x7f
    }
    
    pub(super) fn decode_funct3(&self, inst_bin: u32) -> u32 {
        (inst_bin >> 12) & 0x7
    }
    
    pub(super) fn decode_funct7(&self, inst_bin: u32) -> u32 {
        (inst_bin >> 25) & 0x7f
    }
    
    pub(super) fn decode_quadrant(&self, inst_bin: u16) -> u16 {
        inst_bin & 0x3
    }

    pub(super) fn decode_c_funct3(&self, inst_bin: u16) -> u16 {
        (inst_bin >> 13) & 0x7
    }
}
