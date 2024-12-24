// 定义生成寄存器枚举的宏
macro_rules! generate_gic_registers {
    (
        // 单个寄存器定义
        singles {
            $(
                $single_name:ident = $single_offset:expr
            ),* $(,)?
        }
        // 范围寄存器定义
        ranges {
            $(
                $range_name:ident = {
                    offset: $range_offset:expr,
                    size: $range_size:expr
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum GicRegister {
            // 生成单个寄存器变体
            $(
                $single_name,
            )*
            // 生成范围寄存器变体（带索引）
            $(
                $range_name(u32),
            )*
        }

        impl GicRegister {

            // 从地址转换为寄存器枚举
            pub fn from_addr(addr: u32) -> Option<Self> {
                match addr {
                    // 匹配单个寄存器
                    $(
                        addr if addr == $single_offset => Some(Self::$single_name),
                    )*
                    // 匹配范围寄存器
                    $(
                        addr if addr >= $range_offset && addr < $range_offset + ($range_size * 4) => {
                            let idx = (addr - $range_offset) / 4;
                            if idx < $range_size {
                                Some(Self::$range_name(idx))
                            } else {
                                None
                            }
                        },
                    )*
                    _ => None,
                }
            }
        }
    };
}

// 使用宏生成具体的寄存器定义
generate_gic_registers! {
    singles {
        // 分发器控制寄存器
        GicdCtlr = 0x0000,
        // 分发器类型寄存器
        GicdTyper = 0x0004,
        // 分发器实现识别寄存器
        GicdIidr = 0x0008,
        // 分发器状态寄存器
        GicdStatusr = 0x0010,
    }
    ranges {
        // 中断组寄存器
        GicdIgroupr = {
            offset: 0x0080,
            size: 32
        },
        // 中断使能设置寄存器
        GicdIsenabler = {
            offset: 0x0100,
            size: 32
        },
        // 中断使能清除寄存器
        GicdIcenabler = {
            offset: 0x0180,
            size: 32
        },
        // 中断pending设置寄存器
        GicdIspendr = {
            offset: 0x0200,
            size: 32
        },
        GicdIcpendr = {
            offset: 0x0280,
            size: 32
        },
        GicdIsactiver = {
            offset: 0x0300,
            size: 32
        },
        GicdIcactiver = {
            offset: 0x0380,
            size: 32
        },
        GicdIpriorityr = {
            offset: 0x0400,
            size: 256
        },
        GicdItargetsr = {
            offset: 0x0800,
            size: 256
        },
        GicdIcfgr = {
            offset: 0x0c00,
            size: 64
        },
        GicdPpisr = {
            offset: 0x0d00,
            size: 32
        },
        GicdSpisr = {
            offset: 0x0d04,
            size: 32
        },
        GicdNsacr = {
            offset: 0x0e00,
            size: 32
        },
        GicdSgir = {
            offset: 0x0f00,
            size: 32
        },
        GicdCpendsgir = {
            offset: 0x0f10,
            size: 32
        },
        GicdSpendsgir = {
            offset: 0x0f20,
            size: 32
        },
    }
}
