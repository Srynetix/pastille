// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/cpu/instruction.spec.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! instruction.spec.ts: one instruction at a time, on a CPU with no
//! peripherals, as avr8js runs them; then the instructions its spec leaves
//! out, and the cases where this port departs from it.

use super::programs::*;
use crate::avr::cpu::{ClockEvent, Cpu, Hook, Hooks};
use crate::avr::instruction::avr_instruction;

const R0: usize = 0;
const R1: usize = 1;
const R2: usize = 2;
const R3: usize = 3;
const R4: usize = 4;
const R5: usize = 5;
const R6: usize = 6;
const R7: usize = 7;
const R8: usize = 8;
const R16: usize = 16;
const R17: usize = 17;
const R18: usize = 18;
const R19: usize = 19;
const R20: usize = 20;
const R21: usize = 21;
const R22: usize = 22;
const R23: usize = 23;
const R24: usize = 24;
const R26: usize = 26;
const R27: usize = 27;
const R31: usize = 31;
const X: usize = 26;
const Y: usize = 28;
const Z: usize = 30;
const RAMPZ: usize = 0x5b;
const EIND: usize = 0x5c;
const SP: usize = 93;
const SPH: usize = 94;
const SREG: usize = 95;

// SREG Bits: I-HSVNZC
const SREG_C: u8 = 0b0000_0001;
const SREG_Z: u8 = 0b0000_0010;
const SREG_N: u8 = 0b0000_0100;
const SREG_V: u8 = 0b0000_1000;
const SREG_S: u8 = 0b0001_0000;
const SREG_H: u8 = 0b0010_0000;
const SREG_T: u8 = 0b0100_0000;
const SREG_I: u8 = 0b1000_0000;

/// `new CPU(new Uint16Array(words))` with `program` at address 0.
fn cpu_sized(words: usize, program: &[u16]) -> Cpu {
    let mut cpu = Cpu::new(vec![0; words], 8192);
    cpu.prog_mem[..program.len()].copy_from_slice(program);
    cpu
}

/// The spec's `beforeEach`: 32K words of flash.
fn load(program: &[u16]) -> Cpu {
    cpu_sized(0x8000, program)
}

fn step(cpu: &mut Cpu) {
    avr_instruction(cpu, &mut ());
}

fn get16(cpu: &Cpu, addr: usize) -> u16 {
    cpu.data16(addr as u16)
}

fn set16(cpu: &mut Cpu, addr: usize, value: u16) {
    cpu.set_data16(addr as u16, value);
}

#[test]
fn adc_r0_r1_when_carry_is_on() {
    let mut cpu = load(ADC_R0_R1);
    cpu.data[R0] = 10;
    cpu.data[R1] = 20;
    cpu.data[SREG] = SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 31);
    assert_eq!(cpu.data[SREG], 0);
}

#[test]
fn adc_r0_r1_when_carry_is_on_and_the_result_overflows() {
    let mut cpu = load(ADC_R0_R1);
    cpu.data[R0] = 10;
    cpu.data[R1] = 245;
    cpu.data[SREG] = SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 0);
    assert_eq!(cpu.data[SREG], SREG_H | SREG_Z | SREG_C);
}

#[test]
fn add_r0_r1_when_result_overflows() {
    let mut cpu = load(ADD_R0_R1);
    cpu.data[R0] = 11;
    cpu.data[R1] = 245;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 0);
    assert_eq!(cpu.data[SREG], SREG_H | SREG_Z | SREG_C);
}

#[test]
fn add_r0_r1_when_carry_is_on() {
    let mut cpu = load(ADD_R0_R1);
    cpu.data[R0] = 11;
    cpu.data[R1] = 244;
    cpu.data[SREG] = SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 255);
    assert_eq!(cpu.data[SREG], SREG_S | SREG_N);
}

#[test]
fn add_r0_r1_when_carry_is_on_and_the_result_overflows() {
    let mut cpu = load(ADD_R0_R1);
    cpu.data[R0] = 11;
    cpu.data[R1] = 245;
    cpu.data[SREG] = SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 0);
    assert_eq!(cpu.data[SREG], SREG_H | SREG_Z | SREG_C);
}

#[test]
fn bclr_2() {
    let mut cpu = load(BCLR_2);
    cpu.data[SREG] = 0xff;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], 0xfb);
}

#[test]
fn bld_r4_7() {
    let mut cpu = load(BLD_R4_7);
    cpu.data[R4] = 0x15;
    cpu.data[SREG] = 0x40;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R4], 0x95);
    assert_eq!(cpu.data[SREG], 0x40);
}

#[test]
fn brbc_0_plus_8_when_c_is_clear() {
    let mut cpu = load(BRBC_0_P8);
    cpu.data[SREG] = SREG_V;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1 + 8 / 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn brbc_0_plus_8_when_c_is_set() {
    let mut cpu = load(BRBC_0_P8);
    cpu.data[SREG] = SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
}

#[test]
fn brbs_3_92_when_v_is_set() {
    let mut cpu = load(BRBS_3_92);
    cpu.data[SREG] = SREG_V;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1 + 92 / 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn brbs_3_minus_4_when_v_is_set() {
    let mut cpu = load(BRBS_3_M4);
    cpu.data[SREG] = SREG_V;
    step(&mut cpu);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.cycles, 3); // 1 for NOP, 2 for BRBS
}

#[test]
fn brbs_3_minus_4_when_v_is_clear() {
    let mut cpu = load(BRBS_3_M4);
    cpu.data[SREG] = 0x0;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
}

#[test]
fn call() {
    let mut cpu = load(CALL_0XB8);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 150;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x5c);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(cpu.data[150], 2); // return addr
    assert_eq!(cpu.data[SP], 148); // SP should be decremented
}

#[test]
fn call_pushes_a_three_byte_return_address_with_more_than_128k_of_flash() {
    let mut cpu = cpu_sized(0x20000, CALL_0XB8);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 150;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x5c);
    assert_eq!(cpu.cycles, 5);
    assert_eq!(cpu.data[150], 2); // return addr
    assert_eq!(cpu.data[SP], 147); // SP should be decremented by 3
}

#[test]
fn cbi_0x0c_5() {
    let mut cpu = load(CBI_0X0C_5);
    cpu.data[0x2c] = 0b1111_1111;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    // avr8js: 1; the datasheet says CBI takes two cycles, as SBI does.
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[0x2c], 0b1101_1111);
}

#[test]
fn cpc_r27_r18() {
    let mut cpu = load(CPC_R27_R18);
    cpu.data[R18] = 0x1;
    cpu.data[R27] = 0x1;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], 0);
}

#[test]
fn cpc_r24_r1_and_set() {
    let mut cpu = load(CPC_R24_R1);
    cpu.data[R1] = 0;
    cpu.data[R24] = 0;
    cpu.data[SREG] = SREG_I | SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], SREG_I | SREG_H | SREG_S | SREG_N | SREG_C);
}

#[test]
fn cpi_r26_0x9() {
    let mut cpu = load(CPI_R26_0X9);
    cpu.data[R26] = 0x8;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], SREG_H | SREG_S | SREG_N | SREG_C);
}

#[test]
fn cpse_r2_r3_when_r2_differs() {
    let mut cpu = load(CPSE_R2_R3);
    cpu.data[R2] = 10;
    cpu.data[R3] = 11;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
}

#[test]
fn cpse_r2_r3_when_r2_equals_r3() {
    let mut cpu = load(CPSE_R2_R3);
    cpu.data[R2] = 10;
    cpu.data[R3] = 10;
    step(&mut cpu);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn cpse_r2_r3_when_equal_and_followed_by_a_two_word_instruction() {
    let mut cpu = load(CPSE_R2_R3_CALL_8);
    cpu.data[R2] = 10;
    cpu.data[R3] = 10;
    step(&mut cpu);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn eicall() {
    let mut cpu = cpu_sized(0x20000, EICALL);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x80;
    cpu.data[EIND] = 1;
    set16(&mut cpu, Z, 0x1234);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x11234);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(cpu.data[SP], 0x80 - 3); // according to datasheet: SP ← SP - 3
    assert_eq!(cpu.data[0x80], 1); // Return address
}

#[test]
fn eijmp() {
    let mut cpu = cpu_sized(0x20000, EIJMP);
    cpu.data[EIND] = 1;
    set16(&mut cpu, Z, 0x1040);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x11040);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn elpm() {
    let mut cpu = cpu_sized(0x20000, ELPM);
    cpu.data[Z] = 0x50;
    cpu.data[RAMPZ] = 0x2;
    cpu.set_prog_byte(0x20050, 0x62); // value to be loaded
    step(&mut cpu);
    assert_eq!(cpu.data[R0], 0x62); // check that value was loaded to r0
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn elpm_r5_z() {
    let mut cpu = cpu_sized(0x20000, ELPM_R5_Z);
    cpu.data[Z] = 0x11;
    cpu.data[RAMPZ] = 0x1;
    cpu.set_prog_byte(0x10011, 0x99); // value to be loaded
    step(&mut cpu);
    assert_eq!(cpu.data[R5], 0x99); // check that value was loaded to r5
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn elpm_r6_z_plus() {
    let mut cpu = cpu_sized(0x20000, ELPM_R6_ZP);
    set16(&mut cpu, Z, 0xffff);
    cpu.data[RAMPZ] = 0x2;
    cpu.set_prog_byte(0x2ffff, 0x22); // value to be loaded
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 3);
    assert_eq!(cpu.data[R6], 0x22); // check that value was loaded to r6
    assert_eq!(get16(&cpu, Z), 0x0); // verify that Z was incremented
    assert_eq!(cpu.data[RAMPZ], 3); // verify that RAMPZ was incremented
}

#[test]
fn elpm_r6_z_plus_clamps_rampz() {
    let mut cpu = cpu_sized(0x20000, ELPM_R6_ZP);
    set16(&mut cpu, Z, 0xffff);
    cpu.data[RAMPZ] = 0x3;
    step(&mut cpu);
    assert_eq!(cpu.data[RAMPZ], 0x0); // verify that RAMPZ was reset to zero
}

#[test]
fn icall() {
    let mut cpu = load(ICALL);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x80;
    set16(&mut cpu, Z, 0x2020);
    step(&mut cpu);
    assert_eq!(cpu.cycles, 3);
    assert_eq!(cpu.pc, 0x2020);
    assert_eq!(cpu.data[0x80], 1); // Return address
    assert_eq!(cpu.data[SP], 0x7e); // SP Should decrement by 2
}

#[test]
fn icall_pushes_a_three_byte_return_address_with_more_than_128k_of_flash() {
    let mut cpu = cpu_sized(0x20000, ICALL);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x80;
    set16(&mut cpu, Z, 0x2020);
    step(&mut cpu);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(cpu.pc, 0x2020);
    assert_eq!(cpu.data[0x80], 1); // Return address
    assert_eq!(cpu.data[SP], 0x7d); // SP Should decrement by 3
}

#[test]
fn ijmp() {
    let mut cpu = load(IJMP);
    set16(&mut cpu, Z, 0x1040);
    step(&mut cpu);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.pc, 0x1040);
}

#[test]
fn in_r5_0xb() {
    let mut cpu = load(IN_R5_0XB);
    cpu.data[0x2b] = 0xaf;
    step(&mut cpu);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.data[R5], 0xaf);
}

#[test]
fn inc_r5() {
    let mut cpu = load(INC_R5);
    cpu.data[R5] = 0x7f;
    step(&mut cpu);
    assert_eq!(cpu.data[R5], 0x80);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], SREG_N | SREG_V);
}

#[test]
fn inc_r5_when_r5_is_0xff() {
    let mut cpu = load(INC_R5);
    cpu.data[R5] = 0xff;
    step(&mut cpu);
    assert_eq!(cpu.data[R5], 0);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], SREG_Z);
}

#[test]
fn jmp_0xb8() {
    let mut cpu = load(JMP_0XB8);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x5c);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn lac_z_r19() {
    let mut cpu = load(LAC_Z_R19);
    cpu.data[R19] = 0x02;
    set16(&mut cpu, Z, 0x100);
    cpu.data[0x100] = 0x96;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R19], 0x96);
    assert_eq!(get16(&cpu, Z), 0x100);
    assert_eq!(cpu.data[0x100], 0x94);
}

#[test]
fn las_z_r17() {
    let mut cpu = load(LAS_Z_R17);
    cpu.data[R17] = 0x11;
    cpu.data[Z] = 0x80;
    cpu.data[0x80] = 0x44;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R17], 0x44);
    assert_eq!(cpu.data[Z], 0x80);
    assert_eq!(cpu.data[0x80], 0x55);
}

#[test]
fn lat_z_r0() {
    let mut cpu = load(LAT_Z_R0);
    cpu.data[R0] = 0x33;
    cpu.data[Z] = 0x80;
    cpu.data[0x80] = 0x66;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 0x66);
    assert_eq!(cpu.data[Z], 0x80);
    assert_eq!(cpu.data[0x80], 0x55);
}

/// The `LD`/`LDD` cases: program, pointer, its value before and after, the
/// address read, the register loaded.
fn check_load(
    program: &[u16],
    pointer: usize,
    before: u8,
    after: u8,
    addr: usize,
    register: usize,
) {
    let mut cpu = load(program);
    cpu.data[addr] = 0x15;
    cpu.data[pointer] = before;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[register], 0x15);
    assert_eq!(cpu.data[pointer], after);
}

#[test]
fn ld_r1_x() {
    check_load(LD_R1_X, X, 0xc0, 0xc0, 0xc0, R1); // verify that X was unchanged
}

#[test]
fn ld_r17_x_plus() {
    check_load(LD_R17_XP, X, 0xc0, 0xc1, 0xc0, R17); // verify that X was incremented
}

#[test]
fn ld_r1_minus_x() {
    check_load(LD_R1_MX, X, 0x99, 0x98, 0x98, R1); // verify that X was decremented
}

#[test]
fn ld_r8_y() {
    check_load(LD_R8_Y, Y, 0xc0, 0xc0, 0xc0, R8); // verify that Y was unchanged
}

#[test]
fn ld_r3_y_plus() {
    check_load(LD_R3_YP, Y, 0xc0, 0xc1, 0xc0, R3); // verify that Y was incremented
}

#[test]
fn ld_r0_minus_y() {
    check_load(LD_R0_MY, Y, 0x99, 0x98, 0x98, R0); // verify that Y was decremented
}

#[test]
fn ldd_r4_y_plus_2() {
    check_load(LDD_R4_YP2, Y, 0x80, 0x80, 0x82, R4); // verify that Y was unchanged
}

#[test]
fn ld_r5_z() {
    check_load(LD_R5_Z, Z, 0xcc, 0xcc, 0xcc, R5); // verify that Z was unchanged
}

#[test]
fn ld_r7_z_plus() {
    check_load(LD_R7_ZP, Z, 0xc0, 0xc1, 0xc0, R7); // verify that Z was incremented
}

#[test]
fn ld_r0_minus_z() {
    check_load(LD_R0_MZ, Z, 0x9f, 0x9e, 0x9e, R0); // verify that Z was decremented
}

#[test]
fn ldd_r15_z_plus_31() {
    check_load(LDD_R15_ZP31, Z, 0x80, 0x80, 0x9f, 15); // verify that Z was unchanged
}

#[test]
fn ldi_r28_0xff() {
    let mut cpu = load(LDI_R28_0XFF);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[Y], 0xff);
}

#[test]
fn lds_r5_0x150() {
    let mut cpu = load(LDS_R5_0X150);
    cpu.data[0x150] = 0x7a;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x2);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[R5], 0x7a);
}

/// The `LPM` cases: program, register loaded, Z after.
fn check_lpm(program: &[u16], register: usize, z_after: u8) {
    let mut cpu = load(program);
    cpu.prog_mem[0x40] = 0xa0;
    cpu.data[Z] = 0x80;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 3);
    assert_eq!(cpu.data[register], 0xa0);
    assert_eq!(cpu.data[Z], z_after);
}

#[test]
fn lpm() {
    check_lpm(LPM, R0, 0x80); // verify that Z was unchanged
}

#[test]
fn lpm_r2_z() {
    check_lpm(LPM_R2_Z, R2, 0x80); // verify that Z was unchanged
}

#[test]
fn lpm_r1_z_plus() {
    check_lpm(LPM_R1_ZP, R1, 0x81); // verify that Z was incremented
}

#[test]
fn lsr_r7() {
    let mut cpu = load(LSR_R7);
    cpu.data[R7] = 0x45;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R7], 0x22);
    assert_eq!(cpu.data[SREG], SREG_S | SREG_V | SREG_C);
}

#[test]
fn mov_r7_r8() {
    let mut cpu = load(MOV_R7_R8);
    cpu.data[R8] = 0x45;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R7], 0x45);
}

#[test]
fn movw_r26_r22() {
    let mut cpu = load(MOVW_R26_R22);
    cpu.data[R22] = 0x45;
    cpu.data[R23] = 0x9a;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[X], 0x45);
    assert_eq!(cpu.data[R27], 0x9a);
}

#[test]
fn mul_r5_r6() {
    let mut cpu = load(MUL_R5_R6);
    cpu.data[R5] = 100;
    cpu.data[R6] = 5;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(get16(&cpu, 0), 500);
    assert_eq!(cpu.data[SREG], 0);
}

#[test]
fn mul_r5_r6_sets_carry_when_numbers_are_big() {
    let mut cpu = load(MUL_R5_R6);
    cpu.data[R5] = 200;
    cpu.data[R6] = 200;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(get16(&cpu, 0), 40000);
    assert_eq!(cpu.data[SREG], SREG_C);
}

#[test]
fn mul_r0_r1_sets_zero() {
    let mut cpu = load(MUL_R0_R1);
    cpu.data[R0] = 0;
    cpu.data[R1] = 9;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(get16(&cpu, 0), 0);
    assert_eq!(cpu.data[SREG], SREG_Z);
}

#[test]
fn muls_r18_r19() {
    let mut cpu = load(MULS_R18_R19);
    cpu.data[R18] = (-5i8) as u8;
    cpu.data[R19] = 100;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(get16(&cpu, 0) as i16, -500);
    assert_eq!(cpu.data[SREG], SREG_C);
}

#[test]
fn mulsu_r16_r17() {
    let mut cpu = load(MULSU_R16_R17);
    cpu.data[R16] = (-5i8) as u8;
    cpu.data[R17] = 200;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(get16(&cpu, 0) as i16, -1000);
    assert_eq!(cpu.data[SREG], SREG_C);
}

#[test]
fn neg_r20() {
    let mut cpu = load(NEG_R20);
    cpu.data[R20] = 0x56;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[20], 0xaa);
    // avr8js: S | N | C. 0 − 0x56 borrows from bit 4, so H is set too, as
    // the datasheet's H = R3 + Rd3 says (0xaa has bit 3 set).
    assert_eq!(cpu.data[SREG], SREG_H | SREG_S | SREG_N | SREG_C);
}

#[test]
fn nop() {
    let mut cpu = load(NOP);
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
}

#[test]
fn out_0x3f_r1() {
    let mut cpu = load(OUT_0X3F_R1);
    cpu.data[R1] = 0x5a;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[0x5f], 0x5a);
}

#[test]
fn pop_r26() {
    let mut cpu = load(POP_R26);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0xff;
    cpu.data[0x100] = 0x1a;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[X], 0x1a);
    assert_eq!(get16(&cpu, SP), 0x100);
}

#[test]
fn push_r11() {
    let mut cpu = load(PUSH_R11);
    cpu.data[11] = 0x2a;
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0xff;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[0xff], 0x2a);
    assert_eq!(get16(&cpu, SP), 0xfe);
}

#[test]
fn rcall_plus_6() {
    let mut cpu = load(RCALL_6);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x80;
    step(&mut cpu);
    assert_eq!(cpu.pc, 4);
    assert_eq!(cpu.cycles, 3);
    assert_eq!(get16(&cpu, 0x80), 1); // RET address
    assert_eq!(cpu.data[SP], 0x7e); // SP should decrement by 2
}

#[test]
fn rcall_minus_4() {
    let mut cpu = load(NOP_RCALL_M4);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x80;
    step(&mut cpu);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.cycles, 4); // 1 for NOP, 3 for RCALL
    assert_eq!(get16(&cpu, 0x80), 2); // RET address
    assert_eq!(cpu.data[SP], 0x7e); // SP should decrement by 2
}

#[test]
fn rcall_pushes_a_three_byte_return_address_with_more_than_128k_of_flash() {
    let mut cpu = cpu_sized(0x20000, RCALL_6);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x80;
    set16(&mut cpu, Z, 0x2020);
    step(&mut cpu);
    assert_eq!(cpu.pc, 4);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(get16(&cpu, 0x80), 1); // RET address
    assert_eq!(cpu.data[SP], 0x7d); // SP Should decrement by 3
}

#[test]
fn ret() {
    let mut cpu = load(RET);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x90;
    cpu.data[0x92] = 16;
    step(&mut cpu);
    assert_eq!(cpu.pc, 16);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(cpu.data[SP], 0x92); // SP should increment by 2
}

#[test]
fn ret_with_more_than_128k_of_flash() {
    let mut cpu = cpu_sized(0x20000, RET);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0x90;
    cpu.data[0x91] = 0x1;
    cpu.data[0x93] = 0x16;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x10016);
    assert_eq!(cpu.cycles, 5);
    assert_eq!(cpu.data[SP], 0x93); // SP should increment by 3
}

#[test]
fn reti() {
    let mut cpu = load(RETI);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0xc0;
    cpu.data[0xc2] = 200;
    step(&mut cpu);
    assert_eq!(cpu.pc, 200);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(cpu.data[SP], 0xc2); // SP should increment by 2
    assert_eq!(cpu.data[SREG], SREG_I);
}

#[test]
fn reti_with_more_than_128k_of_flash() {
    let mut cpu = cpu_sized(0x20000, RETI);
    cpu.data[SPH] = 0;
    cpu.data[SP] = 0xc0;
    cpu.data[0xc1] = 0x1;
    cpu.data[0xc3] = 0x30;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x10030);
    assert_eq!(cpu.cycles, 5);
    assert_eq!(cpu.data[SP], 0xc3); // SP should increment by 3
    assert_eq!(cpu.data[SREG], SREG_I);
}

#[test]
fn rjmp_2() {
    let mut cpu = load(RJMP_2);
    step(&mut cpu);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn ror_r0() {
    let mut cpu = load(ROR_R0);
    cpu.data[R0] = 0x11;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 0x08); // r0 should be right-shifted
    assert_eq!(cpu.data[SREG], SREG_S | SREG_V | SREG_C);
}

#[test]
fn sbc_r0_r1_when_carry_is_on_and_result_overflows() {
    let mut cpu = load(SBC_R0_R1);
    cpu.data[R0] = 0;
    cpu.data[R1] = 10;
    cpu.data[SREG] = SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 245);
    assert_eq!(cpu.data[SREG], SREG_H | SREG_S | SREG_N | SREG_C);
}

#[test]
fn sbci_r23_3() {
    let mut cpu = load(SBCI_R23_3);
    cpu.data[R23] = 3;
    cpu.data[SREG] = SREG_I | SREG_C;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[SREG], SREG_I | SREG_H | SREG_S | SREG_N | SREG_C);
}

#[test]
fn sbi_0x0c_5() {
    let mut cpu = load(SBI_0X0C_5);
    cpu.data[0x2c] = 0b0000_1111;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[0x2c], 0b0010_1111);
}

#[test]
fn sbis_0x0c_5_when_bit_is_clear() {
    let mut cpu = load(SBIS_0X0C_5);
    cpu.data[0x2c] = 0b0000_1111;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
}

#[test]
fn sbis_0x0c_5_when_bit_is_set() {
    let mut cpu = load(SBIS_0X0C_5);
    cpu.data[0x2c] = 0b0010_1111;
    step(&mut cpu);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn sbis_0x0c_5_when_bit_is_set_and_followed_by_a_two_word_instruction() {
    let mut cpu = load(SBIS_0X0C_5_CALL_0XB8);
    cpu.data[0x2c] = 0b0010_1111;
    step(&mut cpu);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 3);
}

/// The `ST`/`STD` cases: program, source register and its value, pointer,
/// its value before and after, the address written.
fn check_store(
    program: &[u16],
    register: usize,
    value: u8,
    pointer: usize,
    before: u8,
    after: u8,
    addr: usize,
) {
    let mut cpu = load(program);
    cpu.data[register] = value;
    cpu.data[pointer] = before;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[addr], value);
    assert_eq!(cpu.data[pointer], after);
}

#[test]
fn st_x_r1() {
    check_store(ST_X_R1, R1, 0x5a, X, 0x9a, 0x9a, 0x9a); // verify that X was unchanged
}

#[test]
fn st_x_plus_r1() {
    check_store(ST_XP_R1, R1, 0x5a, X, 0x9a, 0x9b, 0x9a); // verify that X was incremented
}

#[test]
fn st_minus_x_r17() {
    check_store(ST_MX_R17, R17, 0x88, X, 0x99, 0x98, 0x98); // verify that X was decremented
}

#[test]
fn st_y_r2() {
    check_store(ST_Y_R2, R2, 0x5b, Y, 0x9a, 0x9a, 0x9a); // verify that Y was unchanged
}

#[test]
fn st_y_plus_r1() {
    check_store(ST_YP_R1, R1, 0x5a, Y, 0x9a, 0x9b, 0x9a); // verify that Y was incremented
}

#[test]
fn st_minus_y_r1() {
    check_store(ST_MY_R1, R1, 0x5a, Y, 0x9a, 0x99, 0x99); // verify that Y was decremented
}

#[test]
fn std_y_plus_17_r0() {
    check_store(STD_YP17_R0, R0, 0xba, Y, 0x9a, 0x9a, 0x9a + 17); // verify that Y was unchanged
}

#[test]
fn st_z_r16() {
    check_store(ST_Z_R16, R16, 0xdf, Z, 0x40, 0x40, 0x40); // verify that Z was unchanged
}

#[test]
fn st_z_plus_r0() {
    let mut cpu = load(ST_ZP_R0);
    cpu.data[R0] = 0x55;
    set16(&mut cpu, Z, 0x155);
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[0x155], 0x55);
    assert_eq!(get16(&cpu, Z), 0x156); // verify that Z was incremented
}

#[test]
fn st_minus_z_r16() {
    check_store(ST_MZ_R16, R16, 0x5a, Z, 0xff, 0xfe, 0xfe); // verify that Z was decremented
}

#[test]
fn std_z_plus_1_r0() {
    check_store(STD_ZP1_R0, R0, 0xcc, Z, 0x50, 0x50, 0x51); // verify that Z was unchanged
}

#[test]
fn sts_0x151_r31() {
    let mut cpu = load(STS_0X151_R31);
    cpu.data[R31] = 0x80;
    step(&mut cpu);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.data[0x151], 0x80);
}

#[test]
fn sub_r0_r1_when_result_overflows() {
    let mut cpu = load(SUB_R0_R1);
    cpu.data[R0] = 0;
    cpu.data[R1] = 10;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R0], 246);
    // avr8js: S | N | C. 0 − 10 borrows from bit 4: H, by the datasheet's
    // H = !Rd3·Rr3 + Rr3·R3 + R3·!Rd3, with Rr3 = R3 = 1.
    assert_eq!(cpu.data[SREG], SREG_H | SREG_S | SREG_N | SREG_C);
}

#[test]
fn swap_r1() {
    let mut cpu = load(SWAP_R1);
    cpu.data[R1] = 0xa5;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R1], 0x5a);
}

/// Counts `WDR`s, where the spec spies on `cpu.onWatchdogReset`.
#[derive(Default)]
struct Watchdog {
    resets: usize,
}

impl Hooks for Watchdog {
    fn read(&mut self, cpu: &mut Cpu, _: Hook, addr: u16) -> u8 {
        cpu.data[addr as usize]
    }
    fn write(&mut self, _: &mut Cpu, _: Hook, _: u8, _: u8, _: u16, _: u8) -> bool {
        false
    }
    fn clock_event(&mut self, _: &mut Cpu, _: ClockEvent) {}
    fn watchdog_reset(&mut self, _: &mut Cpu) {
        self.resets += 1;
    }
}

#[test]
fn wdr_calls_the_watchdog_reset_hook() {
    let mut cpu = load(WDR);
    let mut watchdog = Watchdog::default();
    assert_eq!(watchdog.resets, 0);
    avr_instruction(&mut cpu, &mut watchdog);
    assert_eq!(watchdog.resets, 1);
}

#[test]
fn xch_z_r21() {
    let mut cpu = load(XCH_Z_R21);
    cpu.data[R21] = 0xa1;
    cpu.data[Z] = 0x50;
    cpu.data[0x50] = 0xb9;
    step(&mut cpu);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 1);
    assert_eq!(cpu.data[R21], 0xb9);
    assert_eq!(cpu.data[0x50], 0xa1);
}

// ---- Not in avr8js's spec: the instructions it leaves untested.

/// Runs `program` once with registers and SREG set, and checks the
/// registers, SREG, `pc` and cycles after.
fn check(program: &[u16], set: &[(usize, u8)], expect: &[(usize, u8)], pc: u32, cycles: u64) {
    let mut cpu = load(program);
    for &(addr, value) in set {
        cpu.data[addr] = value;
    }
    step(&mut cpu);
    for &(addr, value) in expect {
        assert_eq!(cpu.data[addr], value, "data[{addr}]");
    }
    assert_eq!(cpu.pc, pc, "pc");
    assert_eq!(cpu.cycles, cycles, "cycles");
}

#[test]
fn adiw_r24_1_wraps_to_zero_and_keeps_h() {
    check(
        ADIW_R24_1,
        &[(R24, 0xff), (R24 + 1, 0xff), (SREG, SREG_H)],
        &[(R24, 0), (R24 + 1, 0), (SREG, SREG_H | SREG_Z | SREG_C)],
        1,
        2,
    );
}

#[test]
fn adiw_r26_0x3f_overflows_into_the_sign() {
    check(
        ADIW_R26_0X3F,
        &[(R26, 0xc1), (R27, 0x7f)],
        &[(R26, 0), (R27, 0x80), (SREG, SREG_N | SREG_V)],
        1,
        2,
    );
}

#[test]
fn and_r16_r17_keeps_c_and_clears_v() {
    check(
        AND_R16_R17,
        &[(R16, 0xf0), (R17, 0x8c), (SREG, SREG_V | SREG_C)],
        &[(R16, 0x80), (SREG, SREG_S | SREG_N | SREG_C)],
        1,
        1,
    );
}

#[test]
fn andi_r16_0x0f() {
    check(
        ANDI_R16_0X0F,
        &[(R16, 0xf0)],
        &[(R16, 0), (SREG, SREG_Z)],
        1,
        1,
    );
}

#[test]
fn asr_r16_keeps_the_sign() {
    check(
        ASR_R16,
        &[(R16, 0x81)],
        &[(R16, 0xc0), (SREG, SREG_S | SREG_N | SREG_C)],
        1,
        1,
    );
}

#[test]
fn bset_6() {
    check(BSET_6, &[], &[(SREG, SREG_T)], 1, 1);
}

#[test]
fn bst_r16_3() {
    check(BST_R16_3, &[(R16, 0x08)], &[(SREG, SREG_T)], 1, 1);
    check(
        BST_R16_3,
        &[(R16, 0xf7), (SREG, 0xff)],
        &[(SREG, 0xbf)],
        1,
        1,
    );
}

#[test]
fn com_r16() {
    check(
        COM_R16,
        &[(R16, 0x55), (SREG, SREG_V)],
        &[(R16, 0xaa), (SREG, SREG_S | SREG_N | SREG_C)],
        1,
        1,
    );
}

#[test]
fn cp_r16_r17_leaves_both_registers() {
    check(
        CP_R16_R17,
        &[(R16, 0x10), (R17, 0x20)],
        &[(R16, 0x10), (R17, 0x20), (SREG, SREG_S | SREG_N | SREG_C)],
        1,
        1,
    );
}

#[test]
fn dec_r16_from_0x80_overflows_and_keeps_c() {
    check(
        DEC_R16,
        &[(R16, 0x80), (SREG, SREG_C)],
        &[(R16, 0x7f), (SREG, SREG_S | SREG_V | SREG_C)],
        1,
        1,
    );
}

#[test]
fn eor_r1_r1_clears_it() {
    check(EOR_R1_R1, &[(R1, 0x5a)], &[(R1, 0), (SREG, SREG_Z)], 1, 1);
}

#[test]
fn fmul_r16_r17() {
    // 0.5 × 0.5 = 0.25, in 1.7 format: 0x40 × 0x40 = 0x1000, shifted left.
    check(
        FMUL_R16_R17,
        &[(R16, 0x40), (R17, 0x40)],
        &[(R0, 0x00), (R1, 0x20), (SREG, 0)],
        1,
        2,
    );
    // 0xff × 0xff = 0xfe01: C is its bit 15, before the shift.
    check(
        FMUL_R16_R17,
        &[(R16, 0xff), (R17, 0xff)],
        &[(R0, 0x02), (R1, 0xfc), (SREG, SREG_C)],
        1,
        2,
    );
}

#[test]
fn fmuls_r16_r17() {
    // −0.5 × 0.5 = −0.25: −64 × 64 = −4096, shifted left, 0xe000.
    check(
        FMULS_R16_R17,
        &[(R16, 0xc0), (R17, 0x40)],
        &[(R0, 0x00), (R1, 0xe0), (SREG, SREG_C)],
        1,
        2,
    );
}

#[test]
fn fmulsu_r16_r17() {
    check(
        FMULSU_R16_R17,
        &[(R16, 0xc0), (R17, 0x40)],
        &[(R0, 0x00), (R1, 0xe0), (SREG, SREG_C)],
        1,
        2,
    );
}

#[test]
fn fmulsu_sets_z_on_a_zero_product() {
    // avr8js: Z set on a non-zero product and cleared on zero.
    check(
        FMULSU_R16_R17,
        &[(R16, 0), (R17, 0x40)],
        &[(R0, 0), (R1, 0), (SREG, SREG_Z)],
        1,
        2,
    );
}

#[test]
fn or_r16_r17() {
    check(
        OR_R16_R17,
        &[(R16, 0x0f), (R17, 0xf0)],
        &[(R16, 0xff), (SREG, SREG_S | SREG_N)],
        1,
        1,
    );
}

#[test]
fn ori_r16_0x80() {
    check(
        ORI_R16_0X80,
        &[(R16, 0)],
        &[(R16, 0x80), (SREG, SREG_S | SREG_N)],
        1,
        1,
    );
}

#[test]
fn sbic_0x0c_5() {
    check(SBIC_0X0C_5, &[(0x2c, 0b0000_1111)], &[], 2, 2);
    check(SBIC_0X0C_5, &[(0x2c, 0b0010_1111)], &[], 1, 1);
}

#[test]
fn sbiw_r24_1_borrows_and_keeps_h() {
    // avr8js: clears H and recomputes it; SBIW does not touch it.
    check(
        SBIW_R24_1,
        &[(R24, 0), (R24 + 1, 0), (SREG, SREG_H)],
        &[
            (R24, 0xff),
            (R24 + 1, 0xff),
            (SREG, SREG_H | SREG_S | SREG_N | SREG_C),
        ],
        1,
        2,
    );
    check(SBIW_R24_1, &[(R24, 1)], &[(R24, 0), (SREG, SREG_Z)], 1, 2);
}

#[test]
fn sbrc_r16_3() {
    check(SBRC_R16_3, &[(R16, 0)], &[], 2, 2);
    check(SBRC_R16_3, &[(R16, 0x08)], &[], 1, 1);
}

#[test]
fn sbrs_r16_3() {
    check(SBRS_R16_3, &[(R16, 0)], &[], 1, 1);
    check(SBRS_R16_3, &[(R16, 0x08)], &[], 2, 2);
}

#[test]
fn sbrs_r16_3_skips_a_two_word_instruction() {
    check(SBRS_R16_3_JMP_0XB8, &[(R16, 0x08)], &[], 3, 3);
}

#[test]
fn sleep_spm_and_break_do_nothing() {
    for program in [SLEEP, SPM, SPM_ZP, BREAK] {
        let mut cpu = load(program);
        cpu.data[..32].copy_from_slice(&[0x5a; 32]);
        let before = cpu.data.clone();
        step(&mut cpu);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.cycles, 1);
        assert_eq!(cpu.data, before);
        assert_eq!(cpu.prog_mem[0], program[0]);
    }
}

#[test]
fn subi_r16_1() {
    check(
        SUBI_R16_1,
        &[(R16, 0)],
        &[(R16, 0xff), (SREG, SREG_H | SREG_S | SREG_N | SREG_C)],
        1,
        1,
    );
}

#[test]
fn ror_r16_rotates_the_carry_in() {
    check(
        ROR_R16,
        &[(R16, 0x02), (SREG, SREG_C)],
        &[(R16, 0x81), (SREG, SREG_N | SREG_V)],
        1,
        1,
    );
}

#[test]
fn lsr_r16_to_zero() {
    check(
        LSR_R16,
        &[(R16, 0x01)],
        &[(R16, 0), (SREG, SREG_S | SREG_V | SREG_Z | SREG_C)],
        1,
        1,
    );
}

#[test]
fn ldd_y_plus_0_is_ld_y() {
    // Decode order: avr8js tests LD Y before LDD Y+q, and only takes LDD for
    // q ≠ 0.
    check_load(LDD_R4_YP0, Y, 0x80, 0x80, 0x80, R4);
}

#[test]
fn in_out_sbi_cbi_on_a_plain_register() {
    let mut cpu = load(IO_ROUND_TRIP);
    for _ in 0..IO_ROUND_TRIP_INSTRUCTIONS {
        step(&mut cpu);
    }
    assert_eq!(cpu.data[R17], 0x55);
    assert_eq!(cpu.data[0x25], 0x56);
    assert_eq!(cpu.cycles, 1 + 1 + 1 + 2 + 2);
}

#[test]
fn lds_and_sts_reach_the_ends_of_sram() {
    let mut cpu = load(LDS_STS);
    cpu.data[0x8ff] = 0xa5;
    step(&mut cpu);
    step(&mut cpu);
    assert_eq!(cpu.data[0x100], 0xa5);
    assert_eq!(cpu.pc, 4);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn a_backward_jump_to_zero_loops() {
    let mut cpu = load(LOOP_BACK);
    for _ in 0..LOOP_BACK_INSTRUCTIONS {
        step(&mut cpu);
    }
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn the_program_counter_wraps_at_the_end_of_flash() {
    let mut cpu = cpu_sized(4, &[]);
    for _ in 0..4 {
        step(&mut cpu);
    }
    assert_eq!(cpu.pc, 0);
}

// ---- Not in avr8js's spec: the flags this port computes differently.

#[test]
fn half_carry_is_out_of_bit_3() {
    // A carry or borrow across bit 3 and none at bit 0: avr8js, reading bit
    // 0, says no to each.
    check(
        ADD_R16_R17,
        &[(R16, 0x08), (R17, 0x08)],
        &[(R16, 0x10), (SREG, SREG_H)],
        1,
        1,
    );
    check(
        ADC_R16_R17,
        &[(R16, 0x08), (R17, 0x08), (SREG, SREG_C)],
        &[(R16, 0x11), (SREG, SREG_H)],
        1,
        1,
    );
    check(
        SUB_R16_R17,
        &[(R16, 0x10), (R17, 0x02)],
        &[(R16, 0x0e), (SREG, SREG_H)],
        1,
        1,
    );
    // And the other way round: a borrow at bit 0 and none at bit 3, where
    // avr8js says yes.
    check(
        SUB_R16_R17,
        &[(R16, 0x12), (R17, 0x01)],
        &[(R16, 0x11), (SREG, 0)],
        1,
        1,
    );
}

#[test]
fn sbc_and_cpc_keep_z_through_a_borrow_of_256() {
    // 0 − 255 − 1 is −256: the byte is 0, so Z, set before, stays set.
    check(
        SBC_R16_R17,
        &[(R16, 0), (R17, 0xff), (SREG, SREG_Z | SREG_C)],
        &[(R16, 0), (SREG, SREG_H | SREG_Z | SREG_C)],
        1,
        1,
    );
    check(
        CPC_R16_R17,
        &[(R16, 0), (R17, 0xff), (SREG, SREG_Z | SREG_C)],
        &[(R16, 0), (SREG, SREG_H | SREG_Z | SREG_C)],
        1,
        1,
    );
}

#[test]
fn neg_of_0x80_overflows() {
    // avr8js never sets V on NEG.
    check(
        NEG_R16,
        &[(R16, 0x80)],
        &[(R16, 0x80), (SREG, SREG_N | SREG_V | SREG_C)],
        1,
        1,
    );
    check(NEG_R16, &[(R16, 0)], &[(R16, 0), (SREG, SREG_Z)], 1, 1);
}
