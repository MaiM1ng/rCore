.section .text
.global fpu_enable
fpu_enable:
  csrr t0, sstatus
  li t1, 0x2000
  or t0, t0, t1
  csrw sstatus, t0
  ret
