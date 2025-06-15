use crate::prelude::*;

/// 爆炸类型枚举，表示不同的爆炸效果
pub enum ExplosionKind {
    ShipDead,         // 飞船死亡爆炸
    ShipContact,      // 飞船接触碰撞爆炸
    LaserOnAsteroid,  // 激光击中小行星爆炸
}

/// 生成爆炸事件，携带爆炸类型和位置坐标
#[derive(Event)]
pub struct SpawnExplosionEvent {
    pub kind: ExplosionKind,
    pub x: f32,
    pub y: f32,
}

/// 爆炸特效组件，控制爆炸动画的缩放及生命周期
/// 动画效果是：通过定时器timer在生命周期内，将精灵从start_scale缩放到end_scale
#[derive(Component)]
pub struct Explosion {
    timer: Timer,       // 计时器，控制动画时间
    start_scale: f32,   // 动画开始时的缩放比例
    end_scale: f32,     // 动画结束时的缩放比例
}

/// 爆炸特效插件，负责事件监听和爆炸动画系统注册
pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册爆炸生成事件
            .add_event::<SpawnExplosionEvent>()
            // 注册两个系统：动画播放和事件响应，只在游戏运行状态执行
            .add_systems(
                Update,
                (animate_explosion, catch_explosion_event).run_if(in_state(GameState::Running)),
            );
    }
}

/// 监听爆炸生成事件，生成对应爆炸实体
fn catch_explosion_event(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnExplosionEvent>,
    handles: Res<SpriteAssets>, // 纹理资源句柄
    audios: Res<AudioAssets>,   // 音频资源句柄
) {
    for event in event_reader.read() {
        // 根据爆炸类型匹配不同的纹理、音效、大小、缩放和持续时间参数
        let (texture, sound, start_size, end_scale, duration) = match event.kind {
            ExplosionKind::ShipDead => (
                handles.ship_explosion.clone(),
                audios.ship_explosion.clone(),
                Vec2::new(42., 39.),
                5.,     // 放大5倍
                2.,     // 持续2秒
            ),
            ExplosionKind::ShipContact => (
                handles.ship_contact.clone(),
                audios.ship_contact.clone(),
                Vec2::new(42., 39.),
                2.,     // 放大2倍
                1.,     // 持续1秒
            ),
            ExplosionKind::LaserOnAsteroid => (
                handles.asteroid_explosion.clone(),
                audios.asteroid_explosion.clone(),
                Vec2::new(36., 32.),
                1.5,    // 放大1.5倍
                1.,     // 持续1秒
            ),
        };

        // 生成爆炸实体，附加纹理、位置、爆炸组件、音效等
        commands.spawn((
            Sprite {
                image: texture,
                custom_size: Some(start_size), // 设置精灵大小
                ..default()
            },
            Transform::from_translation(Vec3::new(event.x, event.y, 3.0)), // 设置爆炸位置，z轴在3，确保显示在上层
            Explosion {
                timer: Timer::from_seconds(duration, TimerMode::Once), // 定时器
                start_scale: 1.,    // 初始缩放为1
                end_scale,          // 目标缩放比例
            },
            StateScoped(AppState::Game),  // 状态标记
            AudioPlayer(sound),           // 播放爆炸音效
        ));
    }
}

/// 动画系统，控制爆炸实体的缩放动画
///
/// 逻辑：每帧根据定时器进度线性插值缩放值，定时器结束后销毁实体
fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Explosion)>,
    game_state: Res<State<GameState>>,
) {
    // 如果游戏处于暂停状态，则不更新动画计时器（暂停爆炸动画）
    if game_state.get() != &GameState::Paused {
        let elapsed = time.delta();
        for (entity, mut transform, mut explosion) in query.iter_mut() {
            // 推进爆炸动画计时器
            explosion.timer.tick(elapsed);

            if explosion.timer.finished() {
                // 动画结束，销毁爆炸实体
                commands.entity(entity).despawn();
            } else {
                // 根据计时器进度，计算当前缩放比例，线性插值
                transform.scale = Vec3::splat(
                    explosion.start_scale
                        + (explosion.end_scale - explosion.start_scale)
                            * (explosion.timer.elapsed_secs()
                                / explosion.timer.duration().as_secs_f32()),
                );
            }
        }
    }
}
