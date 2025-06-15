use std::time::Duration;

use crate::prelude::*;

// 玩家初始生命值
pub const START_LIFE: u32 = 3;
// 短暂无敌时间（秒）
const INVINCIBLE_TIME: f32 = 2.0;
// 最大累计无敌时间（秒）
const MAX_INVINCIBLE_TIME: f32 = 5.0;

// 玩家动作分为两种枚举类型
// PlayerAction 用于游戏中玩家飞船的操作，绑定在玩家实体上
// MenuAction（未显示）用于菜单操作，作为全局资源添加
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Forward,
    RotateLeft,
    RotateRight,
    Fire,
}

#[derive(Component)]
pub struct Ship {
    /// 飞船旋转速度，单位：弧度/秒
    pub rotation_speed: f32,
    /// 飞船推力，单位：牛顿
    pub thrust: f32,
    /// 飞船生命值
    pub life: u32,
    /// 炮火自动开火计时器
    pub cannon_timer: Timer,
    /// 控制该飞船的玩家ID（1或2）
    pub player_id: u32,
    /// 被击中后触发的短暂无敌计时器
    pub invincible_timer: Timer,
    /// 无敌时间累计（秒）
    pub invincible_time_secs: f32,
}

pub struct PlayerShipPlugin;

impl Plugin for PlayerShipPlugin {
    fn build(&self, app: &mut App) {
        // 添加玩家输入管理插件，支持 PlayerAction 操作映射
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        // 游戏状态进入 Setup 时生成飞船实体
        app.add_systems(OnEnter(GameState::Setup), spawn_ship)
            // 游戏运行时处理飞船输入、阻尼、计时、无敌颜色闪烁及碰撞检测
            .add_systems(
                Update,
                (
                    ship_input_system,
                    ship_dampening_system,
                    ship_timers_system,
                    ship_invincible_color,
                    ship_asteroid_collision,
                )
                    .run_if(in_state(GameState::Running)),
            );
    }
}

// 标记组件，用于更新喷射粒子特效的速度
#[derive(Component)]
pub struct ExhaustEffect;

// 生成玩家飞船实体
fn spawn_ship(mut commands: Commands, handles: Res<SpriteAssets>) {
    // 定义键盘和手柄按键与玩家动作的映射关系
    let input_map = InputMap::new([
        (PlayerAction::Forward, KeyCode::KeyW),
        (PlayerAction::Forward, KeyCode::ArrowUp),
        (PlayerAction::RotateLeft, KeyCode::KeyA),
        (PlayerAction::RotateLeft, KeyCode::ArrowLeft),
        (PlayerAction::RotateRight, KeyCode::KeyD),
        (PlayerAction::RotateRight, KeyCode::ArrowRight),
        (PlayerAction::Fire, KeyCode::Space),
    ]);
    // 创建无敌计时器，初始状态为已计时完成（无敌关闭）
    let mut invincible_timer = Timer::from_seconds(INVINCIBLE_TIME, TimerMode::Once);
    invincible_timer.tick(Duration::from_secs_f32(INVINCIBLE_TIME));

    commands
        .spawn((
            Name::new("PlayerShip"), // 实体名称
            Sprite {
                image: handles.player_ship.clone(),
                custom_size: Some(Vec2::new(30., 20.)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)), // 初始位置
            Ship {
                rotation_speed: 3.0,       // 旋转速度
                thrust: 300000.0,          // 推力
                life: START_LIFE,          // 生命值
                cannon_timer: Timer::from_seconds(0.2, TimerMode::Once), // 炮火冷却
                player_id: 1,              // 玩家ID
                invincible_timer,          // 无敌计时器
                invincible_time_secs: 0.0, // 累计无敌时间
            },
            StateScoped(AppState::Game),                            // 状态标签
            CollisionLayers::new(GameLayer::Player, [GameLayer::Asteroid]), // 碰撞图层
            CollidingEntities::default(),                           // 碰撞实体列表
            RigidBody::Dynamic,                                     // 物理刚体类型
            Collider::circle(13.5),                                 // 碰撞体为圆形，半径13.5
            ExternalForce::default(),                               // 外力组件
            LinearVelocity::ZERO,                                   // 初始线速度为零
            AngularVelocity::ZERO,                                  // 初始角速度为零
            input_map,                                              // 输入映射
        ))
        .observe(on_ship_damage); // 监听飞船受伤事件
}

// 飞船阻尼系统，逐渐减缓速度（模拟摩擦）
fn ship_dampening_system(
    time: Res<Time>,
    mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<Ship>>,
) {
    for (mut linvel, mut angvel) in query.iter_mut() {
        let elapsed = time.delta_secs();
        angvel.0 *= 0.1f32.powf(elapsed); // 角速度阻尼
        linvel.0 *= 0.4f32.powf(elapsed); // 线速度阻尼
    }
}

// 飞船定时器更新系统，更新炮火和无敌计时器
fn ship_timers_system(time: Res<Time>, mut ship: Query<&mut Ship>) {
    for mut ship in ship.iter_mut() {
        ship.cannon_timer.tick(time.delta());
        ship.invincible_timer.tick(time.delta());
    }
}

#[allow(clippy::type_complexity)]
// 处理玩家输入，控制飞船移动和开火
fn ship_input_system(
    mut laser_spawn_events: EventWriter<LaserSpawnEvent>,
    mut query: Query<(
        &ActionState<PlayerAction>,
        &mut ExternalForce,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &Transform,
        &mut Ship,
    )>,
) {
    for (action_state, mut force, linvel, mut angvel, transform, mut ship) in query.iter_mut() {
        // 判断前进键是否按下
        let thrust = if action_state.pressed(&PlayerAction::Forward) {
            1.0
        } else {
            0.0
        };
        // 判断旋转方向
        let rotation = if action_state.pressed(&PlayerAction::RotateLeft) {
            1
        } else if action_state.pressed(&PlayerAction::RotateRight) {
            -1
        } else {
            0
        };
        // 判断是否开火
        let fire = action_state.pressed(&PlayerAction::Fire);
        // 设定角速度
        if rotation != 0 {
            angvel.0 = rotation as f32 * ship.rotation_speed;
        }
        // 根据旋转和推力设置外力
        force.set_force((transform.rotation * (Vec3::Y * thrust * ship.thrust)).truncate());

        // 如果按下开火且炮火冷却完成，触发激光生成事件
        if fire && ship.cannon_timer.finished() {
            laser_spawn_events.write(LaserSpawnEvent {
                transform: *transform,
                linvel: *linvel,
            });
            ship.cannon_timer.reset(); // 重置冷却计时器
        }
    }
}

// 飞船与小行星碰撞检测系统
fn ship_asteroid_collision(
    mut commands: Commands,
    ship_collisions: Query<(Entity, &CollidingEntities), With<Ship>>,
    is_asteroid: Query<(), With<Asteroid>>,
) {
    for (ship, targets) in ship_collisions.iter() {
        for target in targets.iter() {
            // 飞船与小行星碰撞
            // 小行星不受影响，只有飞船受到伤害
            // 爆炸特效由受伤系统处理
            if is_asteroid.contains(*target) {
                commands.trigger_targets(Damage, ship);
            }
        }
    }
}

// 处理飞船受伤事件
fn on_ship_damage(
    trigger: Trigger<Damage>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut explosion_spawn_events: EventWriter<SpawnExplosionEvent>,
    mut ships: Query<(&mut Ship, &Transform)>,
) {
    let ship_entity = trigger.target();
    let (mut ship, ship_transform) = ships
        .get_mut(ship_entity)
        .expect("Missing Ship and Transform on damage trigger");

    // 只有在无敌计时器结束时才允许扣血
    if ship.invincible_timer.finished() {
        ship.invincible_time_secs = 0.0;
        ship.life -= 1; // 扣除1点生命值
        if ship.life == 0 {
            // 生命归零，生成飞船死亡爆炸特效并销毁实体，切换游戏结束状态
            explosion_spawn_events.write(SpawnExplosionEvent {
                kind: ExplosionKind::ShipDead,
                x: ship_transform.translation.x,
                y: ship_transform.translation.y,
            });
            commands.entity(ship_entity).despawn();
            next_state.set(GameState::Over);
        } else {
            // 生命未归零，生成飞船受伤接触爆炸特效
            explosion_spawn_events.write(SpawnExplosionEvent {
                kind: ExplosionKind::ShipContact,
                x: ship_transform.translation.x,
                y: ship_transform.translation.y,
            });
        }
        // 重置无敌计时器
        ship.invincible_timer.reset();
    }
    // 如果处于无敌状态且累计无敌时间未超过最大值，则续期无敌计时器
    else if ship.invincible_time_secs + ship.invincible_timer.elapsed_secs() < MAX_INVINCIBLE_TIME
    {
        ship.invincible_time_secs += ship.invincible_timer.elapsed_secs();
        ship.invincible_timer.reset();
    }
}

// 飞船与小行星接触后会短暂无敌
// 该系统通过让飞船闪烁红色显示无敌状态
// 通过调整精灵的alpha值实现“闪烁”效果
fn ship_invincible_color(mut ships: Query<(&Ship, &mut Sprite)>) {
    for (ship, mut ship_sprite) in ships.iter_mut() {
        if ship.invincible_timer.finished() {
            // 无敌结束，显示正常颜色
            ship_sprite.color = Color::WHITE;
        } else {
            // 无敌期间，颜色透明度周期性变化，呈闪烁红色效果
            let alpha = (ship.invincible_timer.elapsed_secs() * 2.0) % 1.0;
            ship_sprite.color = Color::srgba(1.0, 0.4, 0.2, alpha);
        }
    }
}
