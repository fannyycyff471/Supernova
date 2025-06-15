use crate::prelude::*;
use bevy_hanabi::prelude::*;

// 这个插件负责在游戏中不同节点添加粒子特效
// 为了方便，在面向 WASM 时可以轻松禁用所有粒子效果
pub struct ParticleEffectsPlugin;

impl Plugin for ParticleEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin) // 添加 Hanabi 粒子系统插件
            .add_systems(Update, add_thrust_particles_to_ship) // 监听新生成的飞船，附加喷射粒子
            .add_systems(
                Update,
                update_thrust_particles.run_if(in_state(GameState::Running)), // 仅在游戏运行时更新喷射粒子
            );
    }
}

// 给每个新创建的飞船添加一个喷射粒子效果
fn add_thrust_particles_to_ship(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>, // 粒子效果资源集合
    added_ships: Query<Entity, Added<Ship>>, // 查询新生成的飞船实体
) {
    for ship_entity in added_ships.iter() {
        // 飞船喷射粒子效果相关设置

        let writer = ExprWriter::new();
        let lifetime = writer.lit(0.1).expr(); // 粒子生命周期固定0.1秒

        // 粒子颜色渐变，从紫色到黄色再到透明
        let mut gradient = Gradient::new();
        gradient.add_key(0.0, Vec4::new(0.5, 0.4, 0.7, 0.8));
        gradient.add_key(0.5, Vec4::new(1.0, 0.8, 0.0, 0.8));
        gradient.add_key(1.0, Vec4::ZERO);

        // 粒子初始位置限制在一个锥形体积内，底面半径2，顶面半径1，高度-5
        let init_pos = SetPositionCone3dModifier {
            height: writer.lit(-5.0).expr(),
            base_radius: writer.lit(2.).expr(),
            top_radius: writer.lit(1.).expr(),
            dimension: ShapeDimension::Volume,
        };

        // 粒子初速度为球形分布，速度范围在100到400之间，中心向量为(0,1,0)
        let init_vel = SetVelocitySphereModifier {
            speed: writer.lit(100.0).uniform(writer.lit(400.0)).expr(),
            center: writer.lit(Vec3::new(0.0, 1.0, 0.0)).expr(),
        };

        // 创建一个名为“Exhaust”的粒子效果资源
        let effect = effects.add(
            EffectAsset::new(16024, SpawnerSettings::once(10.0.into()), writer.finish())
                .with_name("Exhaust")
                .init(init_pos)
                .init(init_vel)
                .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime)) // 设置生命周期
                .render(ColorOverLifetimeModifier {
                    gradient,
                    blend: ColorBlendMode::Overwrite,
                    mask: ColorBlendMask::RGBA,
                }) // 颜色渐变效果
                .render(SizeOverLifetimeModifier {
                    gradient: Gradient::constant(Vec3::splat(2.)),
                    screen_space_size: true,
                }), // 粒子尺寸随生命周期保持恒定，且按屏幕空间渲染
        );

        // 将粒子效果作为飞船的子实体生成，位置稍微偏移
        commands.entity(ship_entity).with_children(|parent| {
            parent.spawn((
                ParticleEffect::new(effect),                    // 粒子效果组件
                Transform::from_translation(Vec3::new(0.0, -4.0, 10.0)), // 位置偏移
                ExhaustEffect,                                  // 标记为喷射效果
            ));
        });
    }
}

// 飞船推动时触发粒子生成
fn update_thrust_particles(
    player: Query<(&ActionState<PlayerAction>, &Children), Changed<ActionState<PlayerAction>>>, // 监听玩家动作状态和子实体
    mut exhaust_effect: Query<&mut EffectSpawner, With<ExhaustEffect>>, // 查询喷射粒子生成器组件
) {
    for (action_state, children) in player.iter() {
        // 如果按下了“前进”按键
        if action_state.pressed(&PlayerAction::Forward) {
            for child in children.iter() {
                // 重置喷射粒子生成器，开始发射新粒子
                if let Ok(mut initializers) = exhaust_effect.get_mut(child) {
                    initializers.reset();
                }
            }
        }
    }
}
