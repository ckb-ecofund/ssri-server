// refer to https://github.com/nervosnetwork/ckb-vm/blob/develop/examples/ckb-vm-runner.rs

use std::sync::{Arc, Mutex};

use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::registers::{A0, A7};
use ckb_vm::{Bytes, Memory, Register, SupportMachine, Syscalls};

use crate::error::Error;

#[derive(Clone)]
struct Context {
    content: Arc<Mutex<Option<Bytes>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(None)),
        }
    }
}

impl<M: SupportMachine<REG = u64>> Syscalls<M> for Context {
    fn initialize(&mut self, _machine: &mut M) -> Result<(), ckb_vm::error::Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut M) -> Result<bool, ckb_vm::error::Error> {
        match machine.registers()[A7].to_u64() {
            2041 => machine.set_register(A0, u64::MAX),
            2103 => {
                let mut addr = machine.registers()[A0].to_u64();
                let mut buffer = Vec::new();

                loop {
                    let byte = machine.memory_mut().load8(&M::REG::from_u64(addr))?.to_u8();
                    if byte == 0 {
                        break;
                    }
                    buffer.push(byte);
                    addr += 1;
                }

                *self.content.clone().lock().unwrap() = Some(buffer.into());
            }
            _ => return Ok(false),
        };

        Ok(true)
    }
}

pub fn execute_riscv_binary(code: Bytes, args: Vec<Bytes>) -> Result<Option<Bytes>, Error> {
    let context = Context::new();

    let asm_core = ckb_vm::machine::asm::AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP | ckb_vm::ISA_A,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let core = ckb_vm::DefaultMachineBuilder::new(asm_core)
        .instruction_cycle_func(Box::new(estimate_cycles))
        .syscall(Box::new(context.clone()))
        .build();
    let mut machine = ckb_vm::machine::asm::AsmMachine::new(core);

    machine
        .load_program(&code, &args)
        .map_err(|_| Error::Vm("Failed to load program"))?;
    let error_code = machine
        .run()
        .map_err(|_| Error::Vm("Failed to run program"))?;
    if error_code != 0 {
        return Err(Error::Script(error_code));
    }

    let result = context.content.lock().unwrap().clone();
    Ok(result)
}
