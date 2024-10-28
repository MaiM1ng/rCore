//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::timer::{get_time_ms, get_time_us};
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
    // system time counter
    system_time_stamp: usize,
    // task switch counter
    task_switch_timestamp: usize,
    // total time of task switch
    task_switch_total_time: usize,
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            kernel_time: 0,
            user_time: 0,
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    system_time_stamp: 0,
                    task_switch_timestamp: 0,
                    task_switch_total_time: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        // 更新时间
        self.do_update_system_time_stamp();
        // 初始化时间
        self.do_update_task_switch_timestamp();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("Unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
        println!("[Kernel] task_{} suspended!", current);
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
        println!(
            "[Kernel] task_{} kernel time: {}, user timer: {}",
            current, inner.tasks[current].kernel_time, inner.tasks[current].user_time
        );
        println!("[Kernel] task_{} exited!", current);
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            println!("[Kernel] Run task_{}", next);
            // before this, we should drop local variables that must be dropped manually
            // 更新时间, 用于计算task切换时间
            self.do_update_task_switch_timestamp();
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            let gap = self.do_cal_task_switch_cost();
            println!("[Kernel] Switch task_{}, cost = {} us", current, gap);
            // go back to user mode
        } else {
            println!("All applications completed!");
            shutdown(false);
        }
    }

    fn do_update_current_task_kernel_time(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur_time = get_time_ms();
        let gap = cur_time - inner.system_time_stamp;
        let current_task = inner.current_task;
        inner.tasks[current_task].kernel_time += gap;
        inner.system_time_stamp = cur_time;
    }

    fn do_update_current_task_user_time(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur_time = get_time_ms();
        let gap = cur_time - inner.system_time_stamp;
        let current_task = inner.current_task;
        inner.tasks[current_task].user_time += gap;
        inner.system_time_stamp = cur_time;
    }

    fn do_update_system_time_stamp(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.system_time_stamp = get_time_ms();
    }

    fn do_update_task_switch_timestamp(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.task_switch_timestamp = get_time_us();
    }

    fn do_cal_task_switch_cost(&self) -> usize {
        let mut inner = self.inner.exclusive_access();
        let cur_time_stamp = get_time_us();
        let gap = cur_time_stamp - inner.task_switch_timestamp;
        inner.task_switch_total_time += gap;
        gap
    }
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// 更新当前任务内核态时间
pub fn update_current_task_kernel_time() {
    TASK_MANAGER.do_update_current_task_kernel_time();
}

/// 更新当前任务的用户态时间
pub fn update_current_task_user_time() {
    TASK_MANAGER.do_update_current_task_user_time();
}
