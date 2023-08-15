L2: MOV ax, 0x1234
    MOV bx, 0x5678
    BNE L1

L3: MOV ax, 0x1234
    MOV bx, 0x5678
    BNZ L4

L1: ADD r0, r0, r1

L4: MSR CPSR_c, r0
    MOV pc, lr
