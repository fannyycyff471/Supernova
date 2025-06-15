use crate::prelude::*;

// 定义游戏区域的宽度和高度（屏幕大小）
pub const ARENA_WIDTH: f32 = 1280.0;
pub const ARENA_HEIGHT: f32 = 800.0;

// 表示游戏场景的数据结构（资源）
#[derive(Debug, Resource)]
pub struct Arena {
    // 小行星生成计时器
    pub asteroid_spawn_timer: Timer,
    // 玩家当前得分
    pub score: u32,
}

// 用于物理系统的层分类，区分玩家、激光、和小行星
#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Player,
    Laser,
    Asteroid,
}

// 定义一个“受伤事件”
// Supernova 中我们不需要传递伤害数值或类型，简单表示实体受到伤害即可
#[derive(Event)]
pub struct Damage;

// 定义一个插件，用于设置和管理游戏主场景
pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Setup), spawn_arena) // 进入 Setup 状态时初始化游戏场景
            .add_systems(
                OnEnter(GameState::Running),
                // 游戏开始运行时，启用物理模拟
                |mut physics_time: ResMut<Time<Physics>>| {
                    physics_time.unpause();
                },
            )
            .add_systems(
                OnEnter(GameState::Paused),
                // 游戏暂停时，暂停物理模拟
                |mut physics_time: ResMut<Time<Physics>>| {
                    physics_time.pause();
                },
            )
            // 在游戏运行状态时启用实体位置更新逻辑
            .add_systems(Update, movement.run_if(in_state(GameState::Running)));
    }
}

// 初始化游戏场景资源
fn spawn_arena(mut commands: Commands) {
    commands.insert_resource(Arena {
        // 初始化一个5秒的小行星生成计时器
        asteroid_spawn_timer: Timer::from_seconds(5.0, TimerMode::Once),
        score: 0,
    });

    // 设定重力为 0，避免实体受重力影响
    commands.insert_resource(Gravity::ZERO);
}

// 控制实体在屏幕边缘穿越（“屏幕环绕”效果）
fn movement(mut query: Query<(&LinearVelocity, &mut Position)>) {
    for (linvel, mut position) in query.iter_mut() {
        let mut x = position.x;
        let mut y = position.y;
        let mut updated = false;

        let half_width = ARENA_WIDTH / 2.0;
        let half_height = ARENA_HEIGHT / 2.0;

        // 左右边界判断
        if x < -half_width && linvel.x < 0.0 {
            x = half_width;
            updated = true;
        } else if x > half_width && linvel.x > 0.0 {
            x = -half_width;
            updated = true;
        }

        // 上下边界判断
        if y < -half_height && linvel.y < 0.0 {
            y = half_height;
            updated = true;
        } else if y > half_height && linvel.y > 0.0 {
            y = -half_height;
            updated = true;
        }

        // 如果需要更新位置，就更新坐标
        if updated {
            position.x = x;
            position.y = y;
        }
    }
}
