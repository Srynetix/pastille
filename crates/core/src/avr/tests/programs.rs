// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js: the test programs of src/cpu/instruction.spec.ts,
// src/peripherals/gpio.spec.ts, src/peripherals/timer.spec.ts and
// src/peripherals/adc.spec.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! avr8js's spec programs, assembled.
//!
//! The specs write their programs in assembly and assemble them at run time
//! with avr8js's own assembler. That assembler is not ported: these are its
//! output, produced by running it under Node on each spec's text, with every
//! line of the source beside the words it became. Their names follow the
//! assembly for the one-line programs, and the spec case for the others;
//! `_INSTRUCTIONS` is the spec's `instructionCount`, and a name after the
//! program's is one of its labels, in bytes as avr8js gives them. The few
//! programs below "not in avr8js's spec" were assembled the same way, for the
//! instructions and cases its spec leaves out.
//!
//! Not every count and label is used, so neither is every constant.

#![allow(dead_code)]

// ---- instruction.spec.ts

pub const ADC_R0_R1: &[u16] = &[0x1c01]; // ADC r0, r1

pub const ADD_R0_R1: &[u16] = &[0x0c01]; // ADD r0, r1

pub const BCLR_2: &[u16] = &[0x94a8]; // BCLR 2

pub const BLD_R4_7: &[u16] = &[0xf847]; // BLD r4, 7

pub const BRBC_0_P8: &[u16] = &[0xf420]; // BRBC 0, +8

pub const BRBS_3_92: &[u16] = &[0xf173]; // BRBS 3, 92

pub const BRBS_3_M4: &[u16] = &[0xf3f3]; // BRBS 3, -4

pub const CALL_0XB8: &[u16] = &[0x940e, 0x005c]; // CALL 0xb8

pub const CBI_0X0C_5: &[u16] = &[0x9865]; // CBI 0x0c, 5

pub const CPC_R27_R18: &[u16] = &[0x07b2]; // CPC r27, r18

pub const CPC_R24_R1: &[u16] = &[0x0581]; // CPC r24, r1

pub const CPI_R26_0X9: &[u16] = &[0x30a9]; // CPI r26, 0x9

pub const CPSE_R2_R3: &[u16] = &[0x1023]; // CPSE r2, r3

pub const CPSE_R2_R3_CALL_8: &[u16] = &[
    0x1023, // CPSE r2, r3
    0x940e, 0x0004, // CALL 8
];
pub const CPSE_R2_R3_CALL_8_INSTRUCTIONS: usize = 2;

pub const EICALL: &[u16] = &[0x9519]; // EICALL

pub const EIJMP: &[u16] = &[0x9419]; // EIJMP

pub const ELPM: &[u16] = &[0x95d8]; // ELPM

pub const ELPM_R5_Z: &[u16] = &[0x9056]; // ELPM r5, Z

pub const ELPM_R6_ZP: &[u16] = &[0x9067]; // ELPM r6, Z+

pub const ICALL: &[u16] = &[0x9509]; // ICALL

pub const IJMP: &[u16] = &[0x9409]; // IJMP

pub const IN_R5_0XB: &[u16] = &[0xb05b]; // IN r5, 0xb

pub const INC_R5: &[u16] = &[0x9453]; // INC r5

pub const JMP_0XB8: &[u16] = &[0x940c, 0x005c]; // JMP 0xb8

pub const LAC_Z_R19: &[u16] = &[0x9336]; // LAC Z, r19

pub const LAS_Z_R17: &[u16] = &[0x9315]; // LAS Z, r17

pub const LAT_Z_R0: &[u16] = &[0x9207]; // LAT Z, r0

pub const LD_R1_X: &[u16] = &[0x901c]; // LD r1, X

pub const LD_R17_XP: &[u16] = &[0x911d]; // LD r17, X+

pub const LD_R1_MX: &[u16] = &[0x901e]; // LD r1, -X

pub const LD_R8_Y: &[u16] = &[0x8088]; // LD r8, Y

pub const LD_R3_YP: &[u16] = &[0x9039]; // LD r3, Y+

pub const LD_R0_MY: &[u16] = &[0x900a]; // LD r0, -Y

pub const LDD_R4_YP2: &[u16] = &[0x804a]; // LDD r4, Y+2

pub const LD_R5_Z: &[u16] = &[0x8050]; // LD r5, Z

pub const LD_R7_ZP: &[u16] = &[0x9071]; // LD r7, Z+

pub const LD_R0_MZ: &[u16] = &[0x9002]; // LD r0, -Z

pub const LDD_R15_ZP31: &[u16] = &[0x8cf7]; // LDD r15, Z+31

pub const LDI_R28_0XFF: &[u16] = &[0xefcf]; // LDI r28, 0xff

pub const LDS_R5_0X150: &[u16] = &[0x9050, 0x0150]; // LDS r5, 0x150

pub const LPM: &[u16] = &[0x95c8]; // LPM

pub const LPM_R2_Z: &[u16] = &[0x9024]; // LPM r2, Z

pub const LPM_R1_ZP: &[u16] = &[0x9015]; // LPM r1, Z+

pub const LSR_R7: &[u16] = &[0x9476]; // LSR r7

pub const MOV_R7_R8: &[u16] = &[0x2c78]; // MOV r7, r8

pub const MOVW_R26_R22: &[u16] = &[0x01db]; // MOVW r26, r22

pub const MUL_R5_R6: &[u16] = &[0x9c56]; // MUL r5, r6

pub const MUL_R0_R1: &[u16] = &[0x9c01]; // MUL r0, r1

pub const MULS_R18_R19: &[u16] = &[0x0223]; // MULS r18, r19

pub const MULSU_R16_R17: &[u16] = &[0x0301]; // MULSU r16, r17

pub const NEG_R20: &[u16] = &[0x9541]; // NEG r20

pub const NOP: &[u16] = &[0x0000]; // NOP

pub const OUT_0X3F_R1: &[u16] = &[0xbe1f]; // OUT 0x3f, r1

pub const POP_R26: &[u16] = &[0x91af]; // POP r26

pub const PUSH_R11: &[u16] = &[0x92bf]; // PUSH r11

pub const RCALL_6: &[u16] = &[0xd003]; // RCALL 6

pub const NOP_RCALL_M4: &[u16] = &[
    0x0000, // NOP
    0xdffe, // RCALL -4
];
pub const NOP_RCALL_M4_INSTRUCTIONS: usize = 2;

pub const RET: &[u16] = &[0x9508]; // RET

pub const RETI: &[u16] = &[0x9518]; // RETI

pub const RJMP_2: &[u16] = &[0xc001]; // RJMP 2

pub const ROR_R0: &[u16] = &[0x9407]; // ROR r0

pub const SBC_R0_R1: &[u16] = &[0x0801]; // SBC r0, r1

pub const SBCI_R23_3: &[u16] = &[0x4073]; // SBCI r23, 3

pub const SBI_0X0C_5: &[u16] = &[0x9a65]; // SBI 0x0c, 5

pub const SBIS_0X0C_5: &[u16] = &[0x9b65]; // SBIS 0x0c, 5

pub const SBIS_0X0C_5_CALL_0XB8: &[u16] = &[
    0x9b65, // SBIS 0x0c, 5
    0x940e, 0x005c, // CALL 0xb8
];
pub const SBIS_0X0C_5_CALL_0XB8_INSTRUCTIONS: usize = 2;

pub const ST_X_R1: &[u16] = &[0x921c]; // ST X, r1

pub const ST_XP_R1: &[u16] = &[0x921d]; // ST X+, r1

pub const ST_MX_R17: &[u16] = &[0x931e]; // ST -X, r17

pub const ST_Y_R2: &[u16] = &[0x8228]; // ST Y, r2

pub const ST_YP_R1: &[u16] = &[0x9219]; // ST Y+, r1

pub const ST_MY_R1: &[u16] = &[0x921a]; // ST -Y, r1

pub const STD_YP17_R0: &[u16] = &[0x8a09]; // STD Y+17, r0

pub const ST_Z_R16: &[u16] = &[0x8300]; // ST Z, r16

pub const ST_ZP_R0: &[u16] = &[0x9201]; // ST Z+, r0

pub const ST_MZ_R16: &[u16] = &[0x9302]; // ST -Z, r16

pub const STD_ZP1_R0: &[u16] = &[0x8201]; // STD Z+1, r0

pub const STS_0X151_R31: &[u16] = &[0x93f0, 0x0151]; // STS 0x151, r31

pub const SUB_R0_R1: &[u16] = &[0x1801]; // SUB r0, r1

pub const SWAP_R1: &[u16] = &[0x9412]; // SWAP r1

pub const WDR: &[u16] = &[0x95a8]; // WDR

pub const XCH_Z_R21: &[u16] = &[0x9354]; // XCH Z, r21

// ---- not in avr8js's spec

pub const ADIW_R24_1: &[u16] = &[0x9601]; // ADIW r24, 1

pub const ADIW_R26_0X3F: &[u16] = &[0x96df]; // ADIW r26, 0x3f

pub const AND_R16_R17: &[u16] = &[0x2301]; // AND r16, r17

pub const ANDI_R16_0X0F: &[u16] = &[0x700f]; // ANDI r16, 0x0f

pub const ASR_R16: &[u16] = &[0x9505]; // ASR r16

pub const BSET_6: &[u16] = &[0x9468]; // BSET 6

pub const BST_R16_3: &[u16] = &[0xfb03]; // BST r16, 3

pub const COM_R16: &[u16] = &[0x9500]; // COM r16

pub const CP_R16_R17: &[u16] = &[0x1701]; // CP r16, r17

pub const DEC_R16: &[u16] = &[0x950a]; // DEC r16

pub const EOR_R1_R1: &[u16] = &[0x2411]; // EOR r1, r1

pub const FMUL_R16_R17: &[u16] = &[0x0309]; // FMUL r16, r17

pub const FMULS_R16_R17: &[u16] = &[0x0381]; // FMULS r16, r17

pub const FMULSU_R16_R17: &[u16] = &[0x0389]; // FMULSU r16, r17

pub const OR_R16_R17: &[u16] = &[0x2b01]; // OR r16, r17

pub const ORI_R16_0X80: &[u16] = &[0x6800]; // ORI r16, 0x80

pub const SBIC_0X0C_5: &[u16] = &[0x9965]; // SBIC 0x0c, 5

pub const SBIW_R24_1: &[u16] = &[0x9701]; // SBIW r24, 1

pub const SBRC_R16_3: &[u16] = &[0xfd03]; // SBRC r16, 3

pub const SBRS_R16_3_JMP_0XB8: &[u16] = &[
    0xff03, // SBRS r16, 3
    0x940c, 0x005c, // JMP 0xb8
];
pub const SBRS_R16_3_JMP_0XB8_INSTRUCTIONS: usize = 2;

pub const SBRS_R16_3: &[u16] = &[0xff03]; // SBRS r16, 3

pub const SLEEP: &[u16] = &[0x9588]; // SLEEP

pub const SPM: &[u16] = &[0x95e8]; // SPM

pub const SPM_ZP: &[u16] = &[0x95f8]; // SPM Z+

pub const BREAK: &[u16] = &[0x9598]; // BREAK

pub const SUBI_R16_1: &[u16] = &[0x5001]; // SUBI r16, 1

pub const ROR_R16: &[u16] = &[0x9507]; // ROR r16

pub const LSR_R16: &[u16] = &[0x9506]; // LSR r16

pub const ADD_R16_R17: &[u16] = &[0x0f01]; // ADD r16, r17

pub const ADC_R16_R17: &[u16] = &[0x1f01]; // ADC r16, r17

pub const SUB_R16_R17: &[u16] = &[0x1b01]; // SUB r16, r17

pub const SBC_R16_R17: &[u16] = &[0x0b01]; // SBC r16, r17

pub const CPC_R16_R17: &[u16] = &[0x0701]; // CPC r16, r17

pub const NEG_R16: &[u16] = &[0x9501]; // NEG r16

pub const LDD_R4_YP0: &[u16] = &[0x8048]; // LDD r4, Y+0

pub const IO_ROUND_TRIP: &[u16] = &[
    0xe505, // LDI r16, 0x55
    0xb905, // OUT 0x05, r16
    0xb115, // IN r17, 0x05
    0x9a29, // SBI 0x05, 1
    0x9828, // CBI 0x05, 0
];
pub const IO_ROUND_TRIP_INSTRUCTIONS: usize = 5;

pub const LDS_STS: &[u16] = &[
    0x9100, 0x08ff, // LDS r16, 0x8ff
    0x9300, 0x0100, // STS 0x100, r16
];
pub const LDS_STS_INSTRUCTIONS: usize = 2;

pub const LOOP_BACK: &[u16] = &[
    0x0000, // NOP
    0x0000, // NOP
    0xcffd, // RJMP -6
];
pub const LOOP_BACK_INSTRUCTIONS: usize = 3;

// ---- gpio.spec.ts

pub const GPIO_SBI_ON_PIN: &[u16] = &[
    // ; register addresses
    // _REPLACE DDRD, 10
    // _REPLACE PIND, 9
    // _REPLACE PORTD, 11
    // ; Setup
    0xe488, // ldi r24, 0x48
    0xb98a, // out DDRD, r24
    0xb98b, // out PORTD, r24
    // ; Now toggle pin 6 with SBI
    0x9a4e, // sbi PIND, 6
    0x9598, // break
];
pub const GPIO_SBI_ON_PIN_INSTRUCTIONS: usize = 5;

// ---- timer.spec.ts

pub const TIMER_ISSUE_36: &[u16] = &[
    0xe001, // LDI r16, 0x1    ; TCCR0B = 1 << CS00;
    0xbd05, // OUT 0x25, r16
    0xe300, // LDI r16, 0x30   ; TCNT0 <- 0x30
    0xbd06, // OUT 0x26, r16
    0x0000, // NOP
    0xb516, // IN r17, 0x26    ; r17 <- TCNT
];
pub const TIMER_ISSUE_36_INSTRUCTIONS: usize = 6;

pub const TIMER_ISSUE_40: &[u16] = &[
    0xe001, // LDI r16, 0x1      ; TCCR0B = 1 << CS00
    0xbd05, // OUT 0x25, r16
    0xe000, // LDI r16, 0x0      ; TCNT0 <- 0
    0xbd06, // OUT 0x26, r16
    0x0000, // NOP
    0x9010, 0x0046, // LDS r1, 0x46      ; r1 <- TCNT0 (2 cycles)
];
pub const TIMER_ISSUE_40_INSTRUCTIONS: usize = 6;

pub const TIMER_ISSUE_41: &[u16] = &[
    0x0000, // NOP
    0x0000, // NOP
    0x0000, // NOP
    0x0000, // NOP
    0xe001, // LDI r16, 0x1    ; TCCR2B = 1 << CS20;
    0x9300, 0x00b1, // STS 0xb1, r16   ; Should start counting after this line
    0x0000, // NOP
    0x9110, 0x00b2, // LDS r17, 0xb2   ; TCNT should equal 2 at this point
];
pub const TIMER_ISSUE_41_INSTRUCTIONS: usize = 8;

pub const TIMER_ISSUE_72: &[u16] = &[
    0x2411, // EOR r1, r1      ; r1 = 0;
    0xe001, // LDI r16, 0x1    ; TCCR2B = 1 << CS20;
    0x9300, 0x00b1, // STS 0xb1, r16   ; Should start counting after this instruction,
    0x9210, 0x00b1, // STS 0xb1, r1    ; and stop counting *after* this one.
    0x0000, // NOP
    0x9110,
    0x00b2, // LDS r17, 0xb2   ; TCNT2 should equal 2 at this point (not counting the NOP)
];
pub const TIMER_ISSUE_72_INSTRUCTIONS: usize = 6;

pub const FAST_PWM_SET_CLEAR: &[u16] = &[
    0xef0c, // LDI r16, 0xfc   ; TCNT0 = 0xfc;
    0xbd06, // OUT 0x26, r16
    0xef0e, // LDI r16, 0xfe   ; OCR0A = 0xfe;
    0xbd07, // OUT 0x27, r16
    // ; WGM: Fast PWM, enable OC0A mode 3 (set on Compare Match, clear on Bottom)
    0xec03, // LDI r16, 0xc3   ; TCCR0A = (1 << COM0A1) | (1 << COM0A0) | (1 << WGM01) | (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe001, // LDI r16, 0x1    ; TCCR0B = 1 << CS00;
    0xbd05, // OUT 0x25, r16
    0x0000, // NOP             ; TCNT is now 0xfd
    // beforeMatch:
    0x0000, // NOP             ; TCNT is now 0xfe (Compare Match)
    // afterMatch:
    0x0000, // NOP             ; TCNT is now 0xff
    // beforeBottom:
    0x0000, // NOP             ; TCNT is now 0x00 (BOTTOM)
    // afterBottom:
    0x0000, // NOP
];
pub const FAST_PWM_SET_CLEAR_INSTRUCTIONS: usize = 13;
pub const FAST_PWM_SET_CLEAR_BEFORE_MATCH: u32 = 18;
pub const FAST_PWM_SET_CLEAR_AFTER_MATCH: u32 = 20;
pub const FAST_PWM_SET_CLEAR_BEFORE_BOTTOM: u32 = 22;
pub const FAST_PWM_SET_CLEAR_AFTER_BOTTOM: u32 = 24;

pub const FAST_PWM_TOGGLE: &[u16] = &[
    0xef0c, // LDI r16, 0xfc   ; TCNT0 = 0xfc;
    0xbd06, // OUT 0x26, r16
    0xef0e, // LDI r16, 0xfe   ; OCR0A = 0xfe;
    0xbd07, // OUT 0x27, r16
    // ; WGM: Fast PWM, enable OC0A mode 1 (Toggle)
    0xe403, // LDI r16, 0x43   ; TCCR0A = (1 << COM0A0) | (1 << WGM01) | (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x09   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0x0000, // NOP             ; TCNT is now 0xfd
    // beforeMatch:
    0x0000, // NOP             ; TCNT is now 0xfe (Compare Match, TOP)
    // afterMatch:
    0x0000, // NOP             ; TCNT is now 0
    // afterOverflow:
    0x0000, // NOP
];
pub const FAST_PWM_TOGGLE_INSTRUCTIONS: usize = 12;
pub const FAST_PWM_TOGGLE_BEFORE_MATCH: u32 = 18;
pub const FAST_PWM_TOGGLE_AFTER_MATCH: u32 = 20;
pub const FAST_PWM_TOGGLE_AFTER_OVERFLOW: u32 = 22;

pub const FAST_PWM_DISCONNECT: &[u16] = &[
    0xef0c, // LDI r16, 0xfc   ; TCNT0 = 0xfc;
    0xbd06, // OUT 0x26, r16
    0xef0e, // LDI r16, 0xfe   ; OCR0A = 0xfe;
    0xbd07, // OUT 0x27, r16
    // ; WGM: Fast PWM mode 7, enable OC0A mode 1 (Toggle)
    0xe403, // LDI r16, 0x43   ; TCCR0A = (1 << COM0A0) | (1 << WGM01) | (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x09   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    // beforeClearWGM02:
    0xe001, // LDI r16, 0x01   ; TCCR0B = (1 << CS00);
    0xbd05, // OUT 0x25, r16
    // afterClearWGM02:
    0x0000, // NOP
];
pub const FAST_PWM_DISCONNECT_INSTRUCTIONS: usize = 11;
pub const FAST_PWM_DISCONNECT_BEFORE_CLEAR_WGM02: u32 = 16;
pub const FAST_PWM_DISCONNECT_AFTER_CLEAR_WGM02: u32 = 20;

pub const PHASE_PWM_UP_DOWN: &[u16] = &[
    0xe003, // LDI r16, 0x3   ; OCR0A = 0x3;   // <- TOP value
    0xbd07, // OUT 0x27, r16
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct, top OCR0A
    0xe001, // LDI r16, 0x1   ; TCCR0A = 1 << WGM00;
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x9   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xe002, // LDI r16, 0x2   ; TCNT0 = 0x2;
    0xbd06, // OUT 0x26, r16
    0xb516, // IN r17, 0x26   ; TCNT0 will be 2
    0xb526, // IN r18, 0x26   ; TCNT0 will be 3
    0xb536, // IN r19, 0x26   ; TCNT0 will be 2
    0xb546, // IN r20, 0x26   ; TCNT0 will be 1
    0xb556, // IN r21, 0x26   ; TCNT0 will be 0
    0xb566, // IN r22, 0x26   ; TCNT0 will be 1 (end of test)
];
pub const PHASE_PWM_UP_DOWN_INSTRUCTIONS: usize = 14;

pub const PHASE_PWM_CLEAR_OC0A: &[u16] = &[
    0xef0e, // LDI r16, 0xfe   ; OCR0A = 0xfe;   // <- TOP value
    0xbd07, // OUT 0x27, r16
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct
    0xe801, // LDI r16, 0x81   ; TCCR0A = (1 << COM0A1) | (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe001, // LDI r16, 0x1   ; TCCR0B = (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xef0d, // LDI r16, 0xfd   ; TCNT0 = 0xfd;
    0xbd06, // OUT 0x26, r16
    0x0000, // NOP   ; TCNT0 will be 0xfe
    0x0000, // NOP   ; TCNT0 will be 0xff
    0x0000, // NOP   ; TCNT0 will be 0xfe again (end of test)
];
pub const PHASE_PWM_CLEAR_OC0A_INSTRUCTIONS: usize = 11;

pub const PHASE_PWM_TOGGLE_OC0A: &[u16] = &[
    0xef0e, // LDI r16, 0xfe   ; OCR0A = 0xfe;   // <- TOP value
    0xbd07, // OUT 0x27, r16
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct (mode 5)
    0xe401, // LDI r16, 0x41   ; TCCR0A = (1 << COM0A0) | (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x09   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xef0d, // LDI r16, 0xfd   ; TCNT0 = 0xfd;
    0xbd06, // OUT 0x26, r16
    // beforeMatch:
    0x0000, // NOP             ; TCNT0 will be 0xfe
    // afterMatch:
    0x0000, // NOP
];
pub const PHASE_PWM_TOGGLE_OC0A_INSTRUCTIONS: usize = 10;
pub const PHASE_PWM_TOGGLE_OC0A_BEFORE_MATCH: u32 = 16;
pub const PHASE_PWM_TOGGLE_OC0A_AFTER_MATCH: u32 = 18;

pub const PHASE_PWM_MODE_1: &[u16] = &[
    0xef0e, // LDI r16, 0xfe   ; OCR0A = 0xfe;   // <- TOP value
    0xbd07, // OUT 0x27, r16
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct (mode 1)
    0xe401, // LDI r16, 0x41   ; TCCR0A = (1 << COM0A0) | (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe001, // LDI r16, 0x01   ; TCCR0B = (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xef0d, // LDI r16, 0xfd   ; TCNT0 = 0xfd;
    0xbd06, // OUT 0x26, r16
];
pub const PHASE_PWM_MODE_1_INSTRUCTIONS: usize = 8;

pub const TIMER_ISSUE_79: &[u16] = &[
    0xe100, // LDI r16, 0x10   ; OCR0A = 0x10;   // <- TOP value
    0xbd07, // OUT 0x27, r16
    // ; Set waveform generation mode (WGM) to normal, enable OC0A (Set on match)
    0xec00, // LDI r16, 0xc0   ; TCCR0A = (1 << COM0A1) | (1 << COM0A0);
    0xbd04, // OUT 0x24, r16
    0xe001, // LDI r16, 0x1    ; TCCR0B = (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xe00f, // LDI r16, 0xf    ; TCNT0 = 0xf;
    0xbd06, // OUT 0x26, r16
    0xc000, // RJMP 1          ; TCNT0 will be 0x11 (RJMP takes 2 cycles)
];
pub const TIMER_ISSUE_79_INSTRUCTIONS: usize = 9;

pub const PHASE_PWM_ISSUE_76: &[u16] = &[
    0xe004, // LDI r16, 0x4   ; OCR0A = 0x4;
    0xbd07, // OUT 0x27, r16
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct
    0xe001, // LDI r16, 0x01   ; TCCR0A = (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x09   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xe000, // LDI r16, 0x0   ; TCNT0 = 0x0;
    0xbd06, // OUT 0x26, r16
    0xe002, // LDI r16, 0x2    ; OCR0A = 0x2; // TCNT0 should read 0x0
    0xbd07, // OUT 0x27, r16   ; // TCNT0 should read 0x1
    0x0000, // NOP             ; // TCNT0 should read 0x2
    0x0000, // NOP             ; // TCNT0 should read 0x3
    0xb516, // IN r17, 0x26    ; R17 = TCNT;  // TCNT0 should read 0x4 (that's old OCR0A / TOP)
    0x0000, // NOP             ; // TCNT0 should read 0x3
    0x0000, // NOP             ; // TCNT0 should read 0x2
    0x0000, // NOP             ; // TCNT0 should read 0x1
    0x0000, // NOP             ; // TCNT0 should read 0x0
    0x0000, // NOP             ; // TCNT0 should read 0x1
    0x0000, // NOP             ; // TCNT0 should read 0x2
    0xb526, // IN r18, 0x26    ; R18 = TCNT; // TCNT0 should read 0x1
];
pub const PHASE_PWM_ISSUE_76_INSTRUCTIONS: usize = 20;

pub const PHASE_PWM_ISSUE_119_TOP_ZERO: &[u16] = &[
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct
    0xe001, // LDI r16, 0x01   ; TCCR0A = (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x09   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xe000, // LDI r16, 0x0   ; TCNT0 = 0x0;
    0xbd06, // OUT 0x26, r16
    0xb516, // IN r17, 0x26    ; R17 = TCNT; // TCNT0 should read 0x0
    0xb526, // IN r18, 0x26    ; R18 = TCNT; // TCNT0 should read 0x0
    0xe002, // LDI r16, 0x2    ; OCR0A = 0x2; // TCNT0 should read 0x1
    0xbd07, // OUT 0x27, r16   ; // TCNT0 should read 0x1
    0x0000, // NOP             ; // TCNT0 should read 0x2
    0xb536, // IN r19, 0x26    ; R19 = TCNT; // TCNT0 should read 0x1
];
pub const PHASE_PWM_ISSUE_119_TOP_ZERO_INSTRUCTIONS: usize = 12;

pub const PHASE_PWM_ISSUE_119_OVERRUN: &[u16] = &[
    // ; Set waveform generation mode (WGM) to PWM, Phase Correct
    0xe001, // LDI r16, 0x01   ; TCCR0A = (1 << WGM00);
    0xbd04, // OUT 0x24, r16
    0xe009, // LDI r16, 0x09   ; TCCR0B = (1 << WGM02) | (1 << CS00);
    0xbd05, // OUT 0x25, r16
    0xef0f, // LDI r16, 0xff   ; TCNT0 = 0xff;
    0xbd06, // OUT 0x26, r16
    0xb516, // IN r17, 0x26    ; R17 = TCNT; // TCNT0 should read 255
];
pub const PHASE_PWM_ISSUE_119_OVERRUN_INSTRUCTIONS: usize = 7;

pub const TIMER1_TOGGLE_OC1B: &[u16] = &[
    // ; Set waveform generation mode (WGM) to Normal, top 0xFFFF
    0xe100, // LDI r16, 0x10   ; TCCR1A = (1 << COM1B0);
    0x9300, 0x0080, // STS 0x80, r16
    0xe001, // LDI r16, 0x1    ; TCCR1B = (1 << CS00);
    0x9300, 0x0081, // STS 0x81, r16
    0xe000, // LDI r16, 0x0    ; OCR1BH = 0x0;
    0x9300, 0x008b, // STS 0x8B, r16
    0xe40a, // LDI r16, 0x4a   ; OCR1BL = 0x4a;
    0x9300, 0x008a, // STS 0x8A, r16
    0xe000, // LDI r16, 0x0    ; TCNT1H = 0x0;
    0x9300, 0x0085, // STS 0x85, r16
    0xe409, // LDI r16, 0x49   ; TCNT1L = 0x49;
    0x9300, 0x0084, // STS 0x84, r16
    0x0000, // NOP   ; TCNT1 will be 0x49
    0x0000, // NOP   ; TCNT1 will be 0x4a
];
pub const TIMER1_TOGGLE_OC1B_INSTRUCTIONS: usize = 14;

pub const TIMER1_TOGGLE_OC1C: &[u16] = &[
    // ; Set waveform generation mode (WGM) to Normal, top 0xFFFF
    0xe004, // LDI r16, 0x04   ; TCCR1A = (1 << COM1C0);
    0x9300, 0x0080, // STS 128, r16
    0xe001, // LDI r16, 0x1    ; TCCR1B = (1 << CS00);
    0x9300, 0x0081, // STS 129, r16
    0xe000, // LDI r16, 0x0    ; OCR1CH = 0x0;
    0x9300, 0x008d, // STS 141, r16
    0xe40a, // LDI r16, 0x4a   ; OCR1C = 0x4a;
    0x9300, 0x008c, // STS 140, r16
    0xe000, // LDI r16, 0x0    ; TCNT1H = 0x0;
    0x9300, 0x0085, // STS 133, r16
    0xe409, // LDI r16, 0x49   ; TCNT1 = 0x49;
    0x9300, 0x0084, // STS 132, r16
    0x0000, // NOP   ; TCNT1 will be 0x49
    0x0000, // NOP   ; TCNT1 will be 0x4a
];
pub const TIMER1_TOGGLE_OC1C_INSTRUCTIONS: usize = 14;

pub const TIMER1_ISSUE_76: &[u16] = &[
    0xe000, // LDI r16, 0x0    ; OCR1AH = 0x0;
    0x9300, 0x0089, // STS 0x89, r16
    0xe004, // LDI r16, 0x4   ; OCR1AL = 0x4;
    0x9300, 0x0088, // STS 0x88, r16
    // ; Set waveform generation mode (WGM) to PWM Phase/Frequency Correct mode (9)
    0xe001, // LDI r16, 0x01   ; TCCR1A = (1 << WGM10);
    0x9300, 0x0080, // STS 0x80, r16
    0xe101, // LDI r16, 0x11   ; TCCR1B = (1 << WGM13) | (1 << CS00);
    0x9300, 0x0081, // STS 0x81, r16
    0xe000, // LDI r16, 0x0    ; TCNT1H = 0x0;
    0x9300, 0x0085, // STS 0x85, r16
    0xe000, // LDI r16, 0x0    ; TCNT1L = 0x0;
    0x9300, 0x0084, // STS 0x84, r16
    0xe008, // LDI r16, 0x8   ; OCR1AL = 0x8; // TCNT1 should read 0x0
    0x9300, 0x0088, // STS 0x88, r16  ; // TCNT1 should read 0x2 (going up)
    0x9110, 0x0084, // LDS r17, 0x84  ; // TCNT1 should read 0x4 (going down)
    0x9120, 0x0084, // LDS r18, 0x84  ; // TCNT1 should read 0x2 (going down)
    0x0000, // NOP            ; // TCNT1 should read 0x0 (going up)
    0x0000, // NOP            ; // TCNT1 should read 0x1 (going up)
    0x0000, // NOP            ; // TCNT1 should read 0x2 (going up)
    0x0000, // NOP            ; // TCNT1 should read 0x3 (going up)
    0x0000, // NOP            ; // TCNT1 should read 0x4 (going up)
    0x0000, // NOP            ; // TCNT1 should read 0x5 (going up)
    0x9130, 0x0084, // LDS r19, 0x84  ; // TCNT1 should read 0x6 (going up)
    0x0000, // NOP            ; // TCNT1 should read 0x8 (going up)
    0x9140, 0x0084, // LDS r20, 0x84  ; // TCNT1 should read 0x7 (going up)
];
pub const TIMER1_ISSUE_76_INSTRUCTIONS: usize = 25;

pub const TIMER1_ISSUE_111: &[u16] = &[
    0x2411, // CLR r1          ; r1 is our zero register
    0xe000, // LDI r16, 0x0    ; OCR1AH = 0x0;
    0x9210, 0x0089, // STS 0x89, r1
    0xe008, // LDI r16, 0x8    ; OCR1AL = 0x8;
    0x9300, 0x0088, // STS 0x88, r16
    // ; Set waveform generation mode (WGM) to PWM Phase/Frequency Correct mode (9)
    0xe001, // LDI r16, 0x01   ; TCCR1A = (1 << WGM10);
    0x9300, 0x0080, // STS 0x80, r16
    0xe101, // LDI r16, 0x11   ; TCCR1B = (1 << WGM13) | (1 << CS00);
    0x9300, 0x0081, // STS 0x81, r16
    0x9210, 0x0085, // STS 0x85, r1    ; TCNT1H = 0x0;
    0x9210, 0x0084, // STS 0x84, r1    ; TCNT1L = 0x0;
    0xe005, // LDI r16, 0x5   ; OCR1AL = 0x5; // TCNT1 should read 0x0
    0x9300, 0x0088, // STS 0x88, r16  ; // TCNT1 should read 0x2 (going up)
    0x9210, 0x0084, // STS 0x84, r1   ; TCNT1L = 0x0;
    0x9110, 0x0084, // LDS r17, 0x84  ; // TCNT1 should read 0x1 (going up)
    0x9120, 0x0084, // LDS r18, 0x84  ; // TCNT1 should read 0x3 (going up)
    0x9130, 0x0084, // LDS r19, 0x84  ; // TCNT1 should read 0x5 (going down)
    0x9140, 0x0084, // LDS r20, 0x84  ; // TCNT1 should read 0x3 (going down)
];
pub const TIMER1_ISSUE_111_INSTRUCTIONS: usize = 18;

// ---- adc.spec.ts

pub const ADC_CONVERSION: &[u16] = &[
    0xe480, // LDI r24, 64     ; channel 0, reference AVCC
    0x9380, 0x007c, // STS 0x7c, r24   ; ADMUX
    0xec87, // LDI r24, 199    ; ADEN | ADSC, prescaler 128
    0x9380, 0x007a, // STS 0x7a, r24   ; ADCSRA
    0x9180, 0x007a, // LDS r24, 0x7a   ; waitComplete:
    0x7480, // ANDI r24, 64
    0xf7e1, // BRNE waitComplete
    0x9100, 0x0078, // LDS r16, 0x78   ; ADCL
    0x9110, 0x0079, // LDS r17, 0x79   ; ADCH
    0x9598, // BREAK
];

pub const ADC_NOT_ENABLED: &[u16] = &[
    0xef0f, // LDI r16, 0xff
    0xef1f, // LDI r17, 0xff
    0xe480, // LDI r24, 64     ; channel 0, reference AVCC
    0x9380, 0x007c, // STS 0x7c, r24   ; ADMUX
    0xe487, // LDI r24, 71     ; ADSC, prescaler 128, not enabled
    0x9380, 0x007a, // STS 0x7a, r24   ; ADCSRA
    0x9180, 0x007a, // LDS r24, 0x7a   ; waitComplete:
    0x7480, // ANDI r24, 64
    0xf7e1, // BRNE waitComplete
    0x9100, 0x0078, // LDS r16, 0x78   ; ADCL
    0x9110, 0x0079, // LDS r17, 0x79   ; ADCH
    0x9598, // BREAK
];
