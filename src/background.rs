use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;
use bevy::sprite::Material2dPlugin;

use crate::prelude::*;

// 定义背景插件，负责在游戏初始化时生成背景并持续更新背景时间
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        // 注册自定义材质插件，用于渲染背景
        app.add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            // 在游戏进入 Setup 状态时生成背景实体
            .add_systems(OnEnter(AppState::Setup), spawn_background)
            // 每帧更新背景材质中的时间变量，用于动态效果
            .add_systems(Update, update_background_time);
    }
}

// 生成背景实体，使用一个拉伸的矩形网格和自定义背景材质
fn spawn_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands.spawn((
        // 创建一个矩形网格（默认单位矩形）
        Mesh2d(meshes.add(Rectangle::default())),
        // 缩放矩形以覆盖整个游戏区域（ARENA_WIDTH x ARENA_HEIGHT）
        Transform::from_scale(Vec3::new(ARENA_WIDTH, ARENA_HEIGHT, 1.0)),
        // 应用自定义背景材质，初始时间为0
        MeshMaterial2d(materials.add(BackgroundMaterial { time: 0.0 })),
    ));
}

// 自定义背景材质结构体，包含一个uniform变量time，表示经过时间
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
struct BackgroundMaterial {
    #[uniform(0)]
    time: f32,
}

// 实现Material2d trait，指定使用的片段着色器文件为 background.wgsl
impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "background.wgsl".into()
    }
}

// 每帧更新背景材质的time字段，用来驱动shader的动画
fn update_background_time(
    time: Res<Time>,
    state: Option<Res<State<GameState>>>,
    mut backgrounds: ResMut<Assets<BackgroundMaterial>>,
) {
    // 只有当游戏状态不是暂停时，才更新背景时间
    if state.is_none() || state.unwrap().get() != &GameState::Paused {
        for (_, background) in backgrounds.iter_mut() {
            background.time += time.delta_secs();
        }
    }
}
