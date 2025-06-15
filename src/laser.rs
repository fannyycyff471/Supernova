use crate::prelude::*;

// 激光生成事件，携带激光生成时的位置和速度信息
#[derive(Event)]
pub struct LaserSpawnEvent {
    // 激光的完整位置（包含平移和旋转）
    pub transform: Transform,
    // 发射激光实体的线速度
    pub linvel: LinearVelocity,
}

#[derive(Component)]
pub struct Laser {
    // 激光存在时间计时器，时间到后销毁激光
    pub despawn_timer: Timer,
}

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaserSpawnEvent>() // 注册激光生成事件
            .add_systems(
                Update,
                (
                    spawn_laser,            // 处理激光生成
                    laser_asteroid_collision, // 处理激光与小行星碰撞
                    laser_timeout_system,   // 激光生命周期计时和销毁
                )
                .run_if(in_state(GameState::Running)), // 仅在游戏运行状态处理
            );
    }
}

// 激光生成系统，根据事件创建激光实体
fn spawn_laser(
    mut commands: Commands,
    mut laser_spawn_events: EventReader<LaserSpawnEvent>,
    handles: Res<SpriteAssets>,  // 纹理资源句柄
    audios: Res<AudioAssets>,    // 音效资源
) {
    for spawn_event in laser_spawn_events.read() {
        let mut transform = spawn_event.transform;
        // 强制激光渲染图层z轴为2.0，确保激光在上层
        transform.translation.z = 2.0;
        // 计算激光速度，考虑发射实体的线速度与激光自身方向速度叠加
        let linvel = LinearVelocity(
            (spawn_event.linvel.0 * Vec2::Y) + (transform.rotation * Vec3::Y * 500.0).truncate(),
        );
        // 激光碰撞体为矩形，宽2.5，高10.0
        let collider = Collider::rectangle(2.5, 10.0);
        // 由于xpbd物理引擎生成的激光没有碰撞质量属性，这里显式添加防止运行时警告
        let mass_properties = MassPropertiesBundle::from_shape(&collider, 1.0);
        commands.spawn((
            Name::new("Laser"),   // 实体名称
            Sprite {
                image: handles.laser.clone(),          // 纹理为激光图像
                custom_size: Some(Vec2::new(5., 20.0)), // 自定义尺寸
                ..default()
            },
            transform,                // 位置和旋转信息
            Laser {
                despawn_timer: Timer::from_seconds(2.0, TimerMode::Once), // 激光存在2秒后自动销毁
            },
            CollisionLayers::new(GameLayer::Laser, [GameLayer::Asteroid]), // 激光碰撞层，能与小行星碰撞
            CollidingEntities::default(), // 当前碰撞实体列表（初始化为空）
            RigidBody::Dynamic,          // 动态刚体
            collider,                   // 碰撞体
            mass_properties,            // 碰撞质量属性
            linvel,                    // 线速度
            Sensor,                    // 传感器，不影响物理碰撞响应
            AudioPlayer(audios.laser_trigger.clone()), // 播放激光发射音效
            StateScoped(AppState::Game), // 游戏状态作用域
        ));
    }
}

// 激光与小行星碰撞处理系统
fn laser_asteroid_collision(
    mut commands: Commands,
    mut explosion_spawn_events: EventWriter<SpawnExplosionEvent>, // 触发爆炸事件写入器
    laser_collisions: Query<(Entity, &CollidingEntities), With<Laser>>, // 查询所有激光实体及其碰撞目标
    is_asteroid: Query<(), With<Asteroid>>, // 查询是否为小行星
    transforms: Query<&Transform>,          // 查询实体变换组件
) {
    for (laser, targets) in laser_collisions.iter() {
        for target in targets.iter() {
            // 如果激光碰撞对象是小行星
            if is_asteroid.contains(*target) {
                // 触发小行星受到伤害事件
                commands.trigger_targets(Damage, *target);
                // 获取激光位置，用于生成爆炸效果
                let laser_transform = transforms
                    .get(laser)
                    .expect("Missing transform for the laser");
                // 生成激光击中小行星爆炸特效事件
                explosion_spawn_events.write(SpawnExplosionEvent {
                    kind: ExplosionKind::LaserOnAsteroid,
                    x: laser_transform.translation.x,
                    y: laser_transform.translation.y,
                });
                // 销毁激光实体
                commands.entity(laser).despawn();
            }
        }
    }
}

// 激光超时销毁系统，基于激光的计时器判断是否销毁
fn laser_timeout_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Laser)>,
) {
    for (entity, mut laser) in query.iter_mut() {
        laser.despawn_timer.tick(time.delta()); // 计时器递增
        if laser.despawn_timer.finished() {
            // 计时结束，销毁激光实体
            commands.entity(entity).despawn();
        }
    }
}
