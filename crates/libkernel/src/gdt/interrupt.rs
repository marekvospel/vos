use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::println;

pub(crate) extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub(crate) extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    panic!("EXCEPTION: PAGE FAULT\n{:#?}\n{:#?}", stack_frame, err_code);
}

pub(crate) extern "x86-interrupt" fn double_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("DOUBLE FAULT ({error_code}): \n{:#?}", stack_frame)
}
