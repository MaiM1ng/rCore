/// kernel interrupt check
static mut KERNEL_INTERRUPT_TRIGGERED: bool = false;

use core::ptr::addr_of_mut;

/// 检查是否触发过内核中断
pub fn check_kernel_interrupt() -> bool {
    unsafe { (addr_of_mut!(KERNEL_INTERRUPT_TRIGGERED) as *mut bool).read_volatile() }
}

/// 标记内核中断触发
pub fn mark_kernel_interrupt_triggered() {
    unsafe {
        (addr_of_mut!(KERNEL_INTERRUPT_TRIGGERED) as *mut bool).write_volatile(true);
    }
}
