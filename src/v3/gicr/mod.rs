//! GICv3 Redistributor (GICR) 实现
//!
//! GICR 负责管理每个 CPU 私有的中断（SGI 0-15 和 PPI 16-31）。
//! 在虚拟化环境中，每个 vCPU 都需要一个独立的 GICR 实例。

use alloc::vec::Vec;
use core::ptr::NonNull;

use tock_registers::interfaces::{Readable, ReadWriteable, Writeable};
use vdev_if::{IrqNum, MmioRegion, VirtDeviceOp};

use crate::{IrqChip, Trigger, VGicConfig};

pub mod reg;

// GICv3 私有中断范围常量
const SGI_START: usize = 0;   // SGI (软件生成中断) 起始号
const SGI_END: usize = 15;    // SGI 结束号
const PPI_START: usize = 16;  // PPI (私有外设中断) 起始号
const PPI_END: usize = 31;    // PPI 结束号

pub struct Gicr {
    vmmio_base: MmioRegion,   // GICR_BASE 的 MMIO 区域
    vmmio_sgi: MmioRegion,    // GICR_SGI 的 MMIO 区域
    vcpu_id: usize,           // 关联的 vCPU ID
    reg_base: NonNull<reg::GicrBlockBase>,
    reg_sgi: NonNull<reg::GicrBlockSgi>,
    irqchip: IrqChip,         // 用于读取 PPI 配置
    virt_irqs: Vec<IrqNum>,   // 虚拟中断列表
}

unsafe impl Send for Gicr {}

impl Gicr {
    pub fn new(
        mmio_base: MmioRegion,
        mmio_sgi: MmioRegion,
        vcpu_id: usize,
        config: &VGicConfig,
        virt_irqs: &[IrqNum],
    ) -> Self {
        let reg_base = NonNull::cast::<reg::GicrBlockBase>(mmio_base.access);
        let reg_sgi = NonNull::cast::<reg::GicrBlockSgi>(mmio_sgi.access);

        let mut s = Self {
            vmmio_base: mmio_base,
            vmmio_sgi: mmio_sgi,
            vcpu_id,
            reg_base,
            reg_sgi,
            irqchip: config.irq_chip.clone(),
            virt_irqs: virt_irqs.to_vec(),
        };
        s.init(config);
        s
    }

    fn reg_base(&self) -> &reg::GicrBlockBase {
        unsafe { self.reg_base.as_ref() }
    }

    fn reg_sgi(&self) -> &reg::GicrBlockSgi {
        unsafe { self.reg_sgi.as_ref() }
    }

    fn init(&mut self, config: &VGicConfig) {
        // 1. 配置 GICR_WAKER（唤醒处理器）
        // 确保处理器处于唤醒状态
        self.reg_base().WAKER.write(
            reg::GICR_WAKER::ProcessorSleep::Awake
                + reg::GICR_WAKER::ChildrenAsleep::ChildAwake,
        );

        // 2. 配置 GICR_TYPER（亲和性 + Last 标志）
        // AFF0 设置为 vCPU ID，表示处理器亲和性
        // Last 标志在最后一个 vCPU 的 GICR 中设置
        let is_last = (self.vcpu_id == config.cpu_num - 1) as u64;
        self.reg_base().TYPER.write(
            reg::GICR_TYPER::AFF0.val(self.vcpu_id as u64)
                + reg::GICR_TYPER::Last.val(is_last),
        );

        // 3. 配置中断分组（Group 1）
        // 参考 KVM 和 GICD 的实现，所有中断配置为 Group 1
        self.reg_sgi().IGROUPR0.set(u32::MAX);

        // 4. 设置优先级（0xA0 默认，与 GICD 和 Linux 惯例一致）
        // 每个 IPRIORITYR 寄存器包含 4 个中断的优先级（每 8 位）
        // 中断 0-31，共 8 个寄存器
        let default_priority: u32 = 0xA0A0A0A0; // 4 个 0xA0 打包
        for i in 0..8 {
            self.reg_sgi().IPRIORITYR[i].set(default_priority);
        }

        // 5. 配置 SGI/PPI 触发模式
        self.init_cfg();

        // 6. 清除所有中断状态
        // 禁用所有中断
        self.reg_sgi().ICENABLER0.set(u32::MAX);
        self.reg_sgi().ICENABLER1.set(u32::MAX);

        // 清除所有挂起状态
        self.reg_sgi().ICPENDR0.set(u32::MAX);
        self.reg_sgi().ICPENDR1.set(u32::MAX);

        // 清除所有活跃状态
        self.reg_sgi().ICACTIVER0.set(u32::MAX);
        self.reg_sgi().ICACTIVER1.set(u32::MAX);

        // 7. 初始化 ID 寄存器
        // PIDR2 = 0x3b 表示 GICv3 兼容实现（与 GICD 一致）
        // 必须为两个区域的 PIDR2 都设置相同的值
        self.reg_base().PIDR2.set(0x3b);
        self.reg_sgi().PIDR2.set(0x3b);

        // 8. 配置 GICR_CTLR
        // 初始状态禁用 LPI（LPI 支持在后续阶段实现）
        self.reg_base()
            .CTLR
            .write(reg::GICR_CTLR::EnableLPI::Disabled);

        debug!(
            "GICR: Initialized for vCPU {}, Last={}",
            self.vcpu_id, is_last
        );
    }

    fn init_cfg(&mut self) {
        // SGI 0-15: 必须为边沿触发（硬件强制要求）
        // ICFGR0 每个中断使用 2 位：0b00=Level, 0b10=Edge
        // SGI 必须配置为 0b10 (边沿触发)
        self.reg_sgi().ICFGR0.set(0xAAAAAAAA); // 0b10_10_..._10 (16 个 SGI)

        // PPI 16-31: 从 irqchip 读取配置
        // ICFGR1 每个中断使用 2 位：0b00=Level, 0b10=Edge
        let mut cfg = 0u32;
        for irq_num in PPI_START..=PPI_END {
            let irq: IrqNum = irq_num.into();

            // 从 irqchip 读取当前中断的触发模式
            let trigger = self.irqchip.get_cfg(irq);

            // 转换为 ICFGR 位值
            let cfg_bits = match trigger {
                Trigger::Level => 0b00,
                Trigger::Edge => 0b10,
            };

            // 计算在 ICFGR1 中的位置
            let bit_pos = irq_num - PPI_START;
            let shift = bit_pos * 2;
            cfg |= cfg_bits << shift;
        }
        self.reg_sgi().ICFGR1.set(cfg);

        debug!(
            "GICR: Initialized ICFGR0={:#010x}, ICFGR1={:#010x}",
            self.reg_sgi().ICFGR0.get(),
            self.reg_sgi().ICFGR1.get()
        );
    }
}

impl VirtDeviceOp for Gicr {
    fn name(&self) -> &str {
        "GICv3 redistributor"
    }

    fn invoke(&mut self) {
        // 定期检查和处理
        // 当前实现为空，后续可以添加：
        // - 检查挂起的中断
        // - 同步虚拟和实际硬件状态
        // - 处理虚拟 LPI 相关逻辑
    }
}
