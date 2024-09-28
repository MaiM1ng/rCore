# CH2: Batch System

## 编程题

### 裸机应用程序: 可以打印调用栈

在user下新建文件，可以模仿ch1，对fp寄存器进行跟踪，代码如下

```rust
#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::arch::asm;

#[inline(never)]
#[no_mangle]
fn show_func_trace() {
    let mut fp: usize;
    unsafe {
        asm!(
            "mv {fp}, s0",
            fp = out(reg) fp,
        );
    }

    while fp != 0 {
        let ra = unsafe { *(fp as *const usize).offset(-1) };
        let old_sp = unsafe { *(fp as *const usize).offset(-2) };

        println!("ra = {:x}", ra);
        println!("old sp = {:x}", old_sp);
        println!("");
        fp = old_sp;
    }
}

#[inline(never)]
#[no_mangle]
fn f3() {
    show_func_trace();
}

#[inline(never)]
#[no_mangle]
fn f2() {
    f3();
}

#[inline(never)]
#[no_mangle]
fn f1() {
    f2();
}

#[no_mangle]
fn main() -> i32 {
    println!("====== Stack trace from chain ======\n");

    f1();

    0
}
```

然后可以对生成的elf文件进行反汇编，查看几个函数的地址与ra是否对应。可能存在尾调用优化的问题，因此实际上只输出了f1和main函数的ra。

## 拓展内核: get_taskinfo

延迟到lab3

## 拓展内核: 系统调用统计

延迟到lab3

## 拓展内核：应用执行完成时间

延迟到lab3

## 拓展内核: 统计异常

延迟到lab3

# 问答题

## 函数调用与系统调用有何区别？

系统调用与函数调用的一个最大区别就是在进行系统调用的时候，会产生CPU特权等级的切换，而函数调用不会产生特权等级切换。具体，函数调用在指令层次，只是进行一个pc的修改和函数上下文的保存与回复，只涉及到CPU控制流的流转。

但是系统调用在执行的时候，统一使用同一个入口，在RV中是`ecall`指令，在执行ecall指令的时候，会进行特权等级的切换，然后保存函数的上下文。根据调用信息进行任务分发，这个过程也被称为异常控制流。异常控制流只能流转到kernel指定的目标，函数控制流可以流转到任何地方。

同时由于入口都为`ecall`, 如果user程序想要向kernel传递信息，必须遵守isa的约定，这也就是系统调用之间的ABI。

可以看到，为了支持现代OS的机制，现代CPU设计都无一例外增加了特权等级以及相应的指令，这是软硬件协同的一个体现。

## 为了方便操作系统处理，Ｍ态软件会将 S 态异常/中断委托给 S 态软件，请指出有哪些寄存器记录了委托信息，rustsbi 委托了哪些异常/中断？（也可以直接给出寄存器的值）

1. mideleg：将中断委托给s态
2. medeleg: 将异常委托给s态

这题想问的应该是U和S的异常会路由到S态软件处理，而不进入M态。那么一般是

1. sstatus: 给出发生异常前的信息
2. sepc: 异常发生前最后一条指令的地址
3. scause: 异常原因
4. stval: trap附加信息
5. stvec: 异常handler 入口

rustsbi委托的异常？

## 如果操作系统以应用程序库的形式存在，应用程序可以通过哪些方式破坏操作系统？

这是LibOs，或者Unikernel的风格。特点是应用程序和内核都处于同一特权级，那么此时 应用程序可以随意访问内核栈和内核地址，对内核的数据进行破坏和攻击。

## 编译器/操作系统/处理器如何合作，可采用哪些方法来保护操作系统不受应用程序的破坏？

处理器提供了硬件上的特权等级安全机制，去保证指定内存、寄存器数据无法被破坏。

操作系统隔离应用程序与内核，使用虚拟内存、内核栈等操作去分离。

## RISC-V处理器的S态特权指令有哪些，其大致含义是什么，有啥作用？

1. `sret`： 从S返回U
2. 访问s态的CSR
3. sfence.vma: 刷新tlb

## RISC-V处理器在用户态执行特权指令后的硬件层面的处理过程是什么？

CPU会在执行特权指令的时候判断当前的特权等级是否允许，如果不允许，那么CPU会产生异常，同时将pc置为stvec，同时发生异常等级切换。然后开始处理异常。

## 操作系统在完成用户态<–>内核态双向切换中的一般处理过程是什么？

cpu切换异常等级，保存用户态上下文，切换到异常处理程序，处理完成后恢复用户态上下文。然后回到应用程序继续执行。

## 程序陷入内核的原因有中断、异常和陷入（系统调用），请问 riscv64 支持哪些中断 / 异常？如何判断进入内核是由于中断还是异常？描述陷入内核时的几个重要寄存器及其值

可以参考scause的最高位是1还是0，如果是1为中断，0为异常。

寄存器参考上述回答.

## 在哪些情况下会出现特权级切换：用户态–>内核态，以及内核态–>用户态？

1. 用户态-> 内核态： 应用程序主动唤起，应用程序违法操作，比如执行了违法指令等。
2. 内核态-> 用户态： 异常返回，切换应用程序等。

## Trap上下文的含义是啥？在本章的操作系统中，Trap上下文的具体内容是啥？如果不进行Trap上下文的保存于恢复，会出现什么情况？

应用程序的现场状态。
Trap上下文是指异常寄存器和通用寄存器。如果不保存，则会无法回到异常发生前的现场。

# 实验: `sys_write`安全检查

参考代码

# 问答作业

## 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。请自行测试这些内容 (运行 Rust 三个 bad 测例 ) ，描述程序出错行为，注明你使用的 sbi 及其版本

rust-sbi: v0.3.1

### 对于使用S态特权指令

日志显示

```cmd
[kernel] Loading app_3
Try to execute privileged instruction in U Mode
```

在u模式执行了一条特权指令，触发了异常。然后rCore kill了程序

### 访问S态寄存器

```cmd
[kernel] Loading app_4
Try to access privileged CSR in U Mode
```

同理在U模式访问了S态CSR，产生异常被rCore Kill了

## 请结合用例理解 trap.S 中两个函数 __alltraps 和__restore 的作用，并回答如下几个问题

### L40：刚进入 __restore 时，a0 代表了什么值。请指出__restore 的两种使用情景

刚进入__restore时，a0代表了函数的返回值和参数，需要根据__restore的使用场景进行讨论。

1. 异常恢复： 在异常恢复时，由于`__alltraps`和`__restore`是两个连着的，所以在调用完`trap_handler`以后会继续执行__restore, 此时a0就是trap_handler的返回值，也就是需要恢复user程序的TrapContext。
2. 执行应用程序：在执行应用程序的时候，是直接构造了一个空的TrapContext，然后复用`__restore`函数做一次App Entry，因此此时a0是传递给`__restore`的参数。

但是二者都是指向TrapContext的指针，即要恢复应用程序的上下文。

### L46-L51：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释

```asm
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

将t0 t1 t2 保存到栈上，然后将t012用作临时寄存器取读取三个csr的值。

其中sscratch是一个中间寄存器，在rCore中存放内核栈的地址。
sepc保存了发生异常的下一条指令的地址
sstatus保存了CPU状态信息，进入用户态时可以根据sstatus恢复到之前的CPU状态。

### L53-L59：为何跳过了 x2 和 x4？

```asm
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```

x2是sp寄存器，x4是线程寄存器。

1. sp，此时在恢复除了x2和x4的寄存器，还需要sp寻址。因为此时sp指向内核栈，需要将内核栈上保存的应用程序上下文恢复，如果在第二步就将sp恢复，那么sp会指向user stack，那么就丢失了上下文信息。
2. x4还用不到

### L63：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp
```

交换`sscratch`和sp的值。
在执行之前，sp指向内核栈，sscratch指向用户栈。
执行以后，sp执行用户栈，sscratch执行内核栈。

### __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

sret指令。

isa定义的，根据sstatus保存的 恢复到异常之前的cpu特权等级。

### L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp
```

l13的是将sscratch指向用户栈，sp指向内核栈。

### 从 U 态进入 S 态是哪一条指令发生的？

ecall指令

## 对于任何中断，__alltraps 中都需要保存所有寄存器吗？你有没有想到一些加速__alltraps 的方法？简单描述你的想法

s寄存器和t寄存器应该都需要保存

但是rv貌似没有异常向量这一说法，因此无论啥中断都要走一次__alltraps，然后进一步分发。
