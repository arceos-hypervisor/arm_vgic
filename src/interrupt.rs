use crate::consts::{PPI_ID_MAX, SGI_ID_MAX, SPI_ID_MAX};
use arm_gicv2::{InterruptType, TriggerMode};
use spin::Mutex;

enum InterruptStatus {
    Inactive,
    Pending,
    Active,
    ActivePending,
}

pub struct Interrupt {
    interrupt_id: u32,
    vcpu_id: u32,
    priority: u32,
    status: InterruptStatus,
    active: bool,
    trigger_mode: TriggerMode,
    interrupt_type: InterruptType,
}

pub struct VgicInt {
    inner: Mutex<Interrupt>,
}

impl VgicInt {
    pub fn new(interrupt_id: u32, vcpu_id: u32) -> Self {
        let interrupt_type = if interrupt_id < SGI_ID_MAX as u32 {
            InterruptType::SGI
        } else if interrupt_id < PPI_ID_MAX as u32 {
            InterruptType::PPI
        } else if interrupt_id < SPI_ID_MAX as u32 {
            InterruptType::SPI
        } else {
            panic!("Invalid interrupt id");
        };
        Self {
            inner: Mutex::new(Interrupt {
                interrupt_id,
                vcpu_id,
                priority: 0,
                status: InterruptStatus::Inactive,
                active: false,
                trigger_mode: TriggerMode::Edge,
                interrupt_type,
            }),
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        let mut interrupt = self.inner.lock();
        interrupt.active = enable;
        // if !gicd.get_enable()
        // gicd.set_enable(self.interrupt_id, enable);
    }

    pub fn get_enable(&self) -> bool {
        self.inner.lock().active
    }

    pub fn set_priority(&mut self, priority: u32) {
        let mut interrupt = self.inner.lock();
        interrupt.priority = priority;
        // gicd.set_priority(self.interrupt_id, priority);
    }

    pub fn get_priority(&self) -> u32 {
        self.inner.lock().priority
    }

    pub fn set_vcpu_id(&mut self, vcpu_id: u32) {
        let mut interrupt = self.inner.lock();
        interrupt.vcpu_id = vcpu_id;
    }

    pub fn get_vcpu_id(&self) -> u32 {
        self.inner.lock().vcpu_id
    }

    pub fn set_status(&mut self, status: InterruptStatus) {
        let mut interrupt = self.inner.lock();
        interrupt.status = status;
    }

    pub fn get_status(&self) -> InterruptStatus {
        self.inner.lock().status
    }

    pub fn set_trigger_mode(&mut self, trigger_mode: TriggerMode) {
        let mut interrupt = self.inner.lock();
        interrupt.trigger_mode = trigger_mode;
    }

    pub fn get_trigger_mode(&self) -> TriggerMode {
        self.inner.lock().trigger_mode
    }

    pub fn get_interrupt_type(&self) -> InterruptType {
        self.inner.lock().interrupt_type
    }

    pub fn inject_irq(&self) {
        // todo!
    }
}
