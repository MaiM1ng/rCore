use core::isize;

use crate::batch::{get_app_address_space, get_user_stack_sp, get_user_stack_sp_size};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // check sp
            let sp_top = get_user_stack_sp();
            let sp_bottom = sp_top - get_user_stack_sp_size();
            let sp_range = sp_bottom..sp_top;
            // check address space
            // 由于rust 需要堆，不能仅仅判断app的text段，需要对整个地址空间做判断
            // let app_as = get_current_app_address_space();
            let app_as = get_app_address_space();
            let app_as_range = app_as.0..app_as.1;

            // write 可以写入和读取这两个范围[sp_buttom, sp_top), [as_buttom, as_top)
            // 不在sp里 也不在地址空间里
            if (!sp_range.contains(&(buf as usize)) || !sp_range.contains(&(buf as usize + len)))
                && (!app_as_range.contains(&(buf as usize))
                    || !app_as_range.contains(&(buf as usize + len)))
            {
                return -1;
            }

            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            // panic!("Unsupported fd in sys_write!");
            -1 as isize
        }
    }
}
