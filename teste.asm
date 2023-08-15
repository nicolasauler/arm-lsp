L2: MOV ax, 0x1234
    MOV bx, 0x5678
    BNE L8

L3: mov ax, 0x1234
    MOV bx, 0x5678
    BNZ L4

L1: ADD r0, r0, r1

L4: MSR CPSR_c, r0
    MOV pc, lr

L5: MOV r0, 0x1234
    MOV r1, 0x5678
    MOV r2, 0x9abc
    MOV r3, 0xdef0
    MOV r4, 0x1234
    MOV r5, 0x5678
    MOV r6, 0x9abc
    MOV r7, 0xdef0
    MOV r8, 0x1234
    MOV r9, 0x5678
    MOV r10, 0x9abc
    MOV r11, 0xdef0
    MOV r12, 0x1234
    MOV r13, 0x5678
    MOV r14, 0x9abc
    MOV r15, 0xdef0
    MOV pc, lr

L6: MOV r0, 0x1234

L7: MOV r0, 0x1234
    MOV r1, 0x5678
    MOV r2, 0x9abc
    MOV r3, 0xdef0
    MOV r4, 0x1234
    MOV r5, 0x5678
    MOV r6, 0x9abc
    MOV r7, 0xdef0
    MOV r8, 0x1234
    MOV r9, 0x5678
    MOV r10, 0x9abc
    MOV r11, 0xdef0
    MOV r12, 0x1234
    MOV r13, 0x5678
    MOV r14, 0x9abc
    MOV r15, 0xdef0
    MOV pc, lr

L8: MOV r0, 0x1234

L9: MOV r0, 0x1234
    MOV r1, 0x5678
    MOV r2, 0x9abc
    MOV r3, 0xdef0
    MOV r4, 0x1234
    MOV r5, 0x5678
    MOV r6, 0x9abc
    MOV r7, 0xdef0
    MOV r8, 0x1234
    MOV r9, 0x5678
    MOV r10, 0x9abc
    MOV r11, 0xdef0
    MOV r12, 0x1234
    MOV r13, 0x5678
    MOV r14, 0x9abc
    MOV r15, 0xdef0
    MOV pc, lr
