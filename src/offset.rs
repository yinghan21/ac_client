///视觉矩阵
pub const VIEW_MATRIX: usize = 0x17DFD0;

///实体列表
pub const ENTITY_LIST: usize = 0x18AC04;

///本地玩家
//

///头部指标的xyz
pub const HEAD_POSITION: usize = 0x4;
///脚部指标的x + 0x4 就是y 再+ 0x4就是x
pub const FEET_POSITION: usize = 0x28;

///血量
pub const HEALTH: usize = 0xEC;

///实体上限32个敌人
pub const MAX_PLAYERS: usize = 32;