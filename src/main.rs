#![allow(clippy::too_many_arguments)]

// 模块声明，分别包含游戏各个子系统
mod arena;
mod assets;
mod asteroid;
mod background;
mod explosion;
mod hud;
mod laser;
mod menu;
mod particle_effects;
mod player_ship;
mod state;

// 预导入模块，方便在其它模块中直接使用这些常用类型和函数
mod prelude {
    pub use crate::arena::*;
    pub use crate::assets::*;
    pub use crate::asteroid::*;
    pub use crate::background::*;
    pub use crate::explosion::*;
    pub use crate::hud::*;
    pub use crate::laser::*;
    pub use crate::menu::*;
    pub use crate::player_ship::*;
    pub use crate::state::*;
    pub use avian2d::prelude::*;
    pub use bevy::prelude::*;
    pub use bevy::reflect::TypePath;
    pub use leafwing_input_manager::prelude::*;
    pub use rand::{thread_rng, Rng};
}

use avian2d::prelude::PhysicsPlugins;
use bevy::{
    remote::{http::RemoteHttpPlugin, RemotePlugin}, // 远程调试插件
    window::WindowResolution,
};

use crate::prelude::*;

// 程序入口函数
fn main() {
    let mut app = App::new();

    // 设置窗口背景色为黑色
    app.insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)));

    // 添加默认插件，并配置主窗口标题和大小
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Kataster".to_string(), // 窗口标题
            resolution: WindowResolution::new(ARENA_WIDTH, ARENA_HEIGHT), // 窗口尺寸
            ..default()
        }),
        ..default()
    }));

    // 仅在调试模式下启用调试辅助插件
    #[cfg(debug_assertions)]
    app.add_plugins(PhysicsDebugPlugin::default())  // 物理调试渲染
        .add_plugins(RemotePlugin::default())       // 远程连接插件（比如VSCode调试器）
        .add_plugins(RemoteHttpPlugin::default());  // HTTP远程连接支持

    // 计算着色器不支持WASM平台，非WASM平台启用粒子特效插件
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(particle_effects::ParticleEffectsPlugin);
    }

    // 添加物理系统和菜单输入管理插件
    app.add_plugins((
        PhysicsPlugins::default(),
        InputManagerPlugin::<MenuAction>::default(),
    ));

    // 添加游戏各功能插件
    app.add_plugins((
        StatesPlugin,       // 游戏状态管理
        AssetsPlugin,       // 资源管理
        ArenaPlugin,        // 竞技场相关逻辑
        PlayerShipPlugin,   // 玩家飞船
        LaserPlugin,        // 激光系统
        AsteroidPlugin,     // 小行星
        HudPlugin,          // HUD界面
        MenuPlugin,         // 菜单界面
        ExplosionPlugin,    // 爆炸特效
        BackgroundPlugin,   // 背景
    ));

    // 进入Setup状态时执行摄像机初始化
    app.add_systems(OnEnter(AppState::Setup), setup_camera);

    // 启动游戏
    app.run();
}

// 摄像机初始化，生成一个2D摄像机实体
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}
