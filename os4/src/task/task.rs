//! Types related to task management
use super::TaskContext;
use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use crate::mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};

/// task control block structure
#[derive(Clone, Copy)]
pub struct CurTaskInfo{
    pub sys_write:  u32,
    pub sys_exit:   u32,
    pub sys_info:   u32,
    pub sys_time:   u32,
    pub sys_yield:  u32,
    pub begin_time: usize,
    pub task_status:TaskStatus,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,//应用的地址空间
    pub trap_cx_ppn: PhysPageNum,//位于应用地址空间次高页的Trap上下文被实际存放在物理页帧的物理页号
    pub base_size: usize,//应用数据的大小
    pub task_info: CurTaskInfo,
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);//解析elf相关信息得到的地址空间信息
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);//找到对应的内黑奴关键地址
        KERNEL_SPACE.lock().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );//将应用的内核栈放到对应的地址空间中
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
            task_info: CurTaskInfo::zero_init(),
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit, 
    Ready,
    Running,
    Exited,
}

impl CurTaskInfo{
    pub fn zero_init() -> Self{
        CurTaskInfo { sys_write: 0, sys_exit: 0, sys_info: 0, sys_time: 0, sys_yield: 0, begin_time: 0, task_status:TaskStatus:: Ready }
    }
}
