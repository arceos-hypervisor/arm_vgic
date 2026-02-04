use core::ptr::NonNull;

use tock_registers::interfaces::Writeable;
use vdev_if::MmioRegion;

mod reg;

pub struct Gicd {
    mmio: MmioRegion,
    reg: NonNull<reg::GicdBlock>,
}

unsafe impl Send for Gicd {}

impl Gicd {
    pub fn new(mmio: MmioRegion) -> Self {
        // 将 MMIO 区域转换为 GicdBlock（64KB 中可以包含多个 GicdBlock 页）
        let reg = NonNull::cast::<reg::GicdBlock>(mmio.access);
        let mut s = Self { reg, mmio };
        s.init();
        s
    }

    fn reg(&self) -> &reg::GicdBlock {
        unsafe { self.reg.as_ref() }
    }

    fn init(&mut self) {
        // GICv3 标准中断数
        const MAX_IRQS: usize = 1020;

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

        // 5. 配置触发方式（所有中断边沿触发）
        // 每个 ICFGR 寄存器包含 16 个中断的配置（每 2 位）
        // 0b10 = 边沿触发, 0b00 = 电平触发
        let edge_triggered: u32 = 0xAAAAAAAA;
        for i in 0..num_cfg_regs.min(64) {
            self.reg().ICFGR[i].set(edge_triggered);
        }

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
        self.reg().PIDR2_P15.set(0x3b);  // 0xFFE8: 客户机访问的地址

        // TODO: 初始化 IROUTER（需要访问 IROUTER_P6 和 IROUTER_P7）
    }
}
