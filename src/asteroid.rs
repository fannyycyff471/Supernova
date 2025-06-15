use crate::prelude::*;
use core::time::Duration;

// 小行星生成事件，携带生成小行星的参数
#[derive(Event)]
pub struct AsteroidSpawnEvent {
    pub size: AsteroidSize, // 小行星大小
    pub x: f32,             // 生成位置X坐标
    pub y: f32,             // 生成位置Y坐标
    pub vx: f32,            // 初始水平速度
    pub vy: f32,            // 初始垂直速度
    pub angvel: f32,        // 初始角速度
}

// 小行星大小枚举
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AsteroidSize {
    Big,
    Medium,
    Small,
}

// 实现显示功能，方便打印调试
impl std::fmt::Display for AsteroidSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AsteroidSize {
    // 根据大小返回摧毁该小行星获得的分数
    pub fn score(&self) -> u32 {
        match self {
            AsteroidSize::Big => 40,
            AsteroidSize::Medium => 20,
            AsteroidSize::Small => 10,
        }
    }

    // 定义摧毁该大小小行星后是否分裂成更小的小行星
    // 返回分裂后小行星大小及其生成半径
    pub fn split(&self) -> Option<(AsteroidSize, f32)> {
        match self {
            AsteroidSize::Big => Some((AsteroidSize::Medium, 20.0)),
            AsteroidSize::Medium => Some((AsteroidSize::Small, 10.0)),
            AsteroidSize::Small => None, // 小的不会再分裂
        }
    }
}

// 小行星组件，绑定实体与大小
#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

// 小行星相关的插件，用于注册事件和系统
pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AsteroidSpawnEvent>() // 注册小行星生成事件
            .add_systems(
                Update,
                (arena_asteroids, spawn_asteroid_event) // 更新时处理小行星逻辑
                    .run_if(in_state(GameState::Running)), // 仅在游戏运行状态执行
            );
    }
}

// 监听小行星生成事件，负责创建小行星实体
fn spawn_asteroid_event(
    mut commands: Commands,
    mut event_reader: EventReader<AsteroidSpawnEvent>,
    handles: Res<SpriteAssets>, // 精灵资源句柄
) {
    for event in event_reader.read() {
        // 根据大小选择对应精灵和碰撞半径
        let (sprite_handle, radius) = match event.size {
            AsteroidSize::Big => (handles.meteor_big.clone(), 101. / 2.0),
            AsteroidSize::Medium => (handles.meteor_med.clone(), 43. / 2.0),
            AsteroidSize::Small => (handles.meteor_small.clone(), 28. / 2.0),
        };
        commands
            .spawn((
                Name::new(format!("Asteroid {}", event.size)), // 实体名字
                Sprite {
                    image: sprite_handle.clone(),
                    ..default()
                },
                Transform::from_translation(Vec3::new(event.x, event.y, 1.0)), // 位置
                Asteroid { size: event.size }, // 添加小行星组件
                StateScoped(AppState::Game),   // 只在游戏状态可见
                CollisionLayers::new(
                    GameLayer::Asteroid, // 小行星的物理层
                    [GameLayer::Asteroid, GameLayer::Player, GameLayer::Laser], // 碰撞检测对象
                ),
                RigidBody::Dynamic,           // 动态刚体，受物理影响
                Collider::circle(radius),     // 碰撞体为圆形
                Restitution::new(0.5),        // 碰撞反弹系数
                LinearVelocity(Vec2::new(event.vx, event.vy)), // 初始线速度
                AngularVelocity(event.angvel),                  // 初始角速度
            ))
            .observe(on_asteroid_damage); // 监听受伤事件触发分裂或销毁
    }
}

// 负责定时生成新小行星
fn arena_asteroids(
    time: Res<Time>,
    mut arena: ResMut<Arena>,
    mut asteroid_spawn_events: EventWriter<AsteroidSpawnEvent>,
    asteroids: Query<&Asteroid>,
) {
    arena.asteroid_spawn_timer.tick(time.delta()); // 计时器滴答
    if arena.asteroid_spawn_timer.finished() {
        arena.asteroid_spawn_timer.reset(); // 重置计时器

        let n_asteroid = asteroids.iter().count(); // 当前小行星数量
        if n_asteroid < 20 {
            // 调整计时器周期，越玩越快（但最小0.1秒）
            let duration = Duration::from_secs_f32(
                (0.8 * arena.asteroid_spawn_timer.duration().as_secs_f32()).max(0.1),
            );
            arena.asteroid_spawn_timer.set_duration(duration);

            let mut rng = thread_rng();
            // 随机选择生成小行星的边界（0=顶部，1=左侧）
            let side = rng.gen_range(0..2u8);
            let (x, y) = match side {
                0 => (
                    rng.gen_range((-ARENA_WIDTH / 2.0)..(ARENA_WIDTH / 2.0)),
                    ARENA_HEIGHT / 2.0,
                ),
                _ => (
                    -ARENA_WIDTH / 2.0,
                    rng.gen_range((-ARENA_HEIGHT / 2.0)..(ARENA_HEIGHT / 2.0)),
                ),
            };
            // 随机初速度和角速度
            let vx = rng.gen_range((-ARENA_WIDTH / 4.0)..(ARENA_WIDTH / 4.0));
            let vy = rng.gen_range((-ARENA_HEIGHT / 4.0)..(ARENA_HEIGHT / 4.0));
            let angvel = rng.gen_range(-10.0..10.0);

            // 触发小行星生成事件
            asteroid_spawn_events.write(AsteroidSpawnEvent {
                size: AsteroidSize::Big,
                x,
                y,
                vx,
                vy,
                angvel,
            });
        }
    }
}

// 小行星受伤事件回调，负责处理分裂与销毁
fn on_asteroid_damage(
    trigger: Trigger<Damage>,       // 触发事件的目标
    mut commands: Commands,          // 实体命令管理
    mut arena: ResMut<Arena>,        // 游戏状态资源
    mut asteroid_spawn_events: EventWriter<AsteroidSpawnEvent>, // 生成事件写入器
    asteroids: Query<(&Asteroid, &Transform, &AngularVelocity)>, // 查询小行星相关组件
) {
    let asteroid_entity = trigger.target(); // 受伤的小行星实体
    let (asteroid, asteroid_transform, asteroid_angvel) = asteroids.get(asteroid_entity).unwrap();

    // 增加分数
    arena.score += asteroid.size.score();

    // 如果小行星能分裂，则生成4个更小的小行星
    if let Some((size, radius)) = asteroid.size.split() {
        let mut rng = thread_rng();
        for i in 0..4 {
            let x_pos = if i % 2 == 0 { 1. } else { -1. };
            let y_pos = if (i / 2) % 2 == 0 { 1. } else { -1. };
            let x = asteroid_transform.translation.x + x_pos * 1.5 * radius;
            let y = asteroid_transform.translation.y + y_pos * 1.5 * radius;
            let vx =
                rng.gen_range((-ARENA_WIDTH / (radius / 4.))..(ARENA_WIDTH / (radius / 4.)));
            let vy =
                rng.gen_range((-ARENA_HEIGHT / (radius / 4.))..(ARENA_HEIGHT / (radius / 4.)));

            // 发送生成新小行星事件
            asteroid_spawn_events.write(AsteroidSpawnEvent {
                size,
                x,
                y,
                vx,
                vy,
                angvel: asteroid_angvel.0,
            });
        }
    }

    // 销毁当前受伤小行星实体
    commands.entity(asteroid_entity).despawn();
}
