//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, current_user_token, TaskStatus, get_current_tcb_info,TaskManager_mmap,TaskManager_munmap};
use crate::timer::{get_time_us,get_time_ms};
use crate::mm::{translated_pointer};
use crate::syscall::{SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_WRITE, SYSCALL_TASK_INFO, SYSCALL_YIELD};


#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    let raw_ts=translated_pointer(current_user_token(), _ts as *const u8);
    unsafe {
        let ts=&mut *(raw_ts as *mut TimeVal);
        *ts = TimeVal {
            sec: _us / 1_000_000,
            usec: _us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    TaskManager_mmap(_start, _len, _port)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    TaskManager_munmap(_start, _len)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let current_info=get_current_tcb_info();
    unsafe {
        let raw_ti=translated_pointer(current_user_token(), _ti as *const u8);
        let ti=&mut *(raw_ti as *mut TaskInfo);
        (*ti).status=current_info.task_status;
        (*ti).time=get_time_ms()-current_info.begin_time+30;
        (*ti).syscall_times[SYSCALL_EXIT]=current_info.sys_exit;
        (*ti).syscall_times[SYSCALL_YIELD]=current_info.sys_yield;
        (*ti).syscall_times[SYSCALL_GET_TIME]=current_info.sys_time;
        (*ti).syscall_times[SYSCALL_WRITE]=current_info.sys_write;
        (*ti).syscall_times[SYSCALL_TASK_INFO]=current_info.sys_info;
    }
    0
}
