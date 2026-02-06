use alloc::vec::Vec;
use core::ptr::NonNull;
use mmio_api::Mmio;

use aarch64_cpu::registers::Readable;
use tock_registers::interfaces::{Debuggable, Writeable};
use vdev_if::{IrqNum, MmioRegion, VirtDeviceOp};

use crate::v3::gicd::reg::{GICD_CTLR, GICD_TYPER};
use crate::{IrqChip, VGicConfig};

mod reg;

// GICv3 中断范围常量
const SPI_START: usize = 32;  // SPI (共享外设中断) 起始号
const SPI_END: usize = 1020;  // SPI 结束号
const NUM_CFG_REGS: usize = 64; // ICFGR 配置寄存器数量

pub struct Gicd {
    vmmio: MmioRegion,
    irqchip: IrqChip,
    reg: NonNull<reg::GicdBlock>,
    virt_irqs: Vec<IrqNum>,
    cfg_shot: Vec<u32>,
}

unsafe impl Send for Gicd {}

impl Gicd {
    pub fn new(mmio: MmioRegion, config: &VGicConfig, virt_irqs: &[IrqNum]) -> Self {
        // 将 MMIO 区域转换为 GicdBlock（64KB 中可以包含多个 GicdBlock 页）
        let reg = NonNull::cast::<reg::GicdBlock>(mmio.access);
        let mut s = Self {
            reg,
            irqchip: config.irq_chip.clone(),
            vmmio: mmio,
            virt_irqs: virt_irqs.to_vec(),
            cfg_shot: vec![],
        };
        s.init(config);
        s
    }

    fn reg(&self) -> &reg::GicdBlock {
        unsafe { self.reg.as_ref() }
    }

    fn init(&mut self, config: &VGicConfig) {
        // GICv3 标准中断数
        const MAX_IRQS: usize = 1020;

        self.reg().TYPER.write(
            GICD_TYPER::ITLinesNumber.val((MAX_IRQS / 32) as u32 - 1)
                + GICD_TYPER::CPUNumber.val(config.cpu_num as _),
        );

        // 计算需要的寄存器数量
        let num_en_regs = (MAX_IRQS + 31) / 32; // ISENABLER/ICENABLER 等
        let num_prio_regs = ((MAX_IRQS * 8) + 31) / 32; // IPRIORITYR
        let num_cfg_regs = ((MAX_IRQS * 2) + 31) / 32; // ICFGR
        let num_grpmod_regs = (MAX_IRQS + 31) / 32; // IGRPMODR
        let num_nsacr_regs = (MAX_IRQS + 15) / 16; // NSACR

        // 1. 配置 GICD_CTLR（参考 KVM：ARE_NS | DS）
        // 使用 tock-registers 位域操作，避免魔法数字
        self.reg()
            .CTLR
            .write(reg::GICD_CTLR::ARE_NS::Enabled + reg::GICD_CTLR::DS::SecurityDisabled);

        // 2. 配置中断分组（参考 KVM：Group 1）
        for i in 0..num_en_regs {
            self.reg().IGROUPR[i].set(u32::MAX); // 所有中断 → Group 1
        }

        // 3. 配置 Group Modifier（Group 1）
        for i in 0..num_grpmod_regs {
            self.reg().IGRPMODR[i].set(0);
        }

        // 4. 设置中断优先级（Linux 惯例：0xA0）
        // 每个 IPRIORITYR 寄存器包含 4 个中断的优先级（每 8 位）
        let default_priority: u32 = 0xA0A0A0A0; // 4 个 0xA0 打包
        for i in 0..num_prio_regs.min(255) {
            self.reg().IPRIORITYR[i].set(default_priority);
        }

        self.init_cfg();

        // 6. 配置访问控制（禁止非安全访问）
        for i in 0..num_nsacr_regs.min(32) {
            self.reg().NSACR[i].set(0);
        }

        // 7. 清除所有中断状态（禁用、清除挂起、清除活跃）
        for i in 0..num_en_regs {
            self.reg().ICENABLER[i].set(u32::MAX); // 禁用所有中断
            self.reg().ICPENDR[i].set(u32::MAX); // 清除挂起状态
            self.reg().ICACTIVER[i].set(u32::MAX); // 清除活跃状态
        }

        // 8. 初始化 ID 寄存器（参考 KVM）
        // PIDR2 返回 0x3b 表示 GICv3 兼容实现
        // 必须为所有 16 个页面的 PIDR2 设置相同的值
        self.reg().PIDR2_P0.set(0x3b);
        self.reg().PIDR2_P1.set(0x3b);
        self.reg().PIDR2_P2.set(0x3b);
        self.reg().PIDR2_P3.set(0x3b);
        self.reg().PIDR2_P4.set(0x3b);
        self.reg().PIDR2_P5.set(0x3b);
        self.reg().PIDR2_P6.set(0x3b);
        self.reg().PIDR2_P7.set(0x3b);
        self.reg().PIDR2_P8.set(0x3b);
        self.reg().PIDR2_P9.set(0x3b);
        self.reg().PIDR2_P10.set(0x3b);
        self.reg().PIDR2_P11.set(0x3b);
        self.reg().PIDR2_P12.set(0x3b);
        self.reg().PIDR2_P13.set(0x3b);
        self.reg().PIDR2_P14.set(0x3b);
        self.reg().PIDR2_P15.set(0x3b); // 0xFFE8: 客户机访问的地址
    }

    fn check_ctrl(&mut self) {}

    fn init_cfg(&mut self) {
        // 初始化配置快照数组
        self.cfg_shot = vec![0u32; NUM_CFG_REGS];

        // 遍历所有 SPI 中断（32-1020）
        for irq_num in SPI_START..SPI_END {
            let irq: IrqNum = irq_num.into();

            // 从 irqchip 读取当前配置
            let trigger = self.irqchip.get_cfg(irq);

            // 计算在 cfg_shot 中的位置
            let reg_idx = irq_num / 16; // 寄存器索引
            let bit_pos = irq_num % 16; // 寄存器内位偏移
            let shift = bit_pos * 2;

            // 转换为 ICFGR 位值并打包
            let cfg_bits = match trigger {
                crate::Trigger::Level => 0b00,
                crate::Trigger::Edge => 0b10,
            };

            self.cfg_shot[reg_idx] |= cfg_bits << shift;
        }

        debug!(
            "GICD: Initialized config snapshot for SPI {}-{}",
            SPI_START, SPI_END
        );
    }

    fn setup_cfg(&mut self) {
        let new_val_ls = self
            .reg()
            .ICFGR
            .iter()
            .map(|r| r.get())
            .collect::<Vec<u32>>();
        // 恢复中断配置寄存器
        for (i, cfg) in self.cfg_shot.iter_mut().enumerate() {
            let new_val = new_val_ls[i];
            if *cfg != new_val {
                let diff = *cfg ^ new_val;
                // 每个 ICFGR 寄存器包含 16 个中断的配置（每 2 位）
                for bit in 0..16 {
                    let shift = bit * 2;
                    if ((diff >> shift) & 0b11) == 0 {
                        continue;
                    }

                    let irq_num = i * 16 + bit;

                    // 跳过 CPU 私有中断（SGI 0-15 和 PPI 16-31）
                    if irq_num < SPI_START {
                        continue;
                    }
                    // 边界检查
                    if irq_num >= SPI_END {
                        continue;
                    }

                    let irq: IrqNum = irq_num.into();

                    if !self.virt_irqs.is_empty() && !self.virt_irqs.contains(&irq) {
                        continue;
                    }

                    let cfg_bits = (new_val >> shift) & 0b11;
                    let trig = if cfg_bits == 0b10 {
                        crate::Trigger::Edge
                    } else {
                        crate::Trigger::Level
                    };
                    debug!("GICD IRQ({irq_num}) cfg changed, set to {:?}", trig);
                    self.irqchip.set_cfg(irq, trig);
                }

                *cfg = new_val;
            }
        }
    }
}

impl VirtDeviceOp for Gicd {
    fn name(&self) -> &str {
        "GICv3 distributor"
    }

    fn invoke(&mut self) {
        self.check_ctrl();
        self.setup_cfg();
    }
}
