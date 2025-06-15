use bevy::ecs::spawn::SpawnIter;

use crate::prelude::*;

// 分数UI组件标记
#[derive(Component)]
pub struct UiScore {}

// 生命值UI组件，min表示该图标代表的最低生命数
#[derive(Component)]
pub struct UiLife {
    pub min: u32,
}

// HUD插件，负责分数和生命值显示
pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (hud_score_system, hud_life_system).run_if(in_state(GameState::Running)),
        )
        // 游戏状态切换到Setup时生成HUD UI
        .add_systems(OnEnter(GameState::Setup), hud_spawn);
    }
}

// 生成HUD界面，包括分数文本和生命值图标
fn hud_spawn(mut commands: Commands, assets: ResMut<UiAssets>) {
    // 分数文本节点
    commands.spawn((
        Node {
            position_type: PositionType::Absolute, // 绝对定位
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexStart,   // 垂直方向顶部对齐
            justify_content: JustifyContent::FlexEnd, // 水平方向右对齐
            flex_direction: FlexDirection::Row,
            ..default()
        },
        StateScoped(AppState::Game), // 只在游戏状态显示
        children![(
            Text::new("0"),             // 初始分数为0
            TextFont {
                font: assets.font.clone(), // 使用UI字体资源
                font_size: 50.0,
                ..default()
            },
            TextColor(Color::srgb_u8(0x00, 0xAA, 0xAA)), // 青绿色文字
            TextLayout::new_with_justify(JustifyText::Right), // 右对齐文本
            Node {
                margin: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                },
                ..default()
            },
            UiScore {}, // 标记为分数UI，方便更新系统识别
        )],
    ));
    // 生命值图标节点
    // 注意：此处不在GameOver状态保存生命图标，简化了生命减少的处理
    let ship_life_image = assets.ship_life.clone(); // 生命图标纹理资源
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,  // 绝对定位
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexStart,    // 垂直方向顶部对齐
            justify_content: JustifyContent::FlexStart, // 水平方向左对齐
            flex_direction: FlexDirection::Row,
            ..default()
        },
        StateScoped(AppState::Game), // 只在游戏状态显示
        // 生成多个生命图标实体，范围是 1 到 START_LIFE
        Children::spawn(SpawnIter((1..(START_LIFE + 1)).map(move |i| {
            (
                ImageNode::new(ship_life_image.clone()), // 生命图标图片
                Node {
                    margin: UiRect {
                        left: Val::Px(10.0),
                        right: Val::Px(10.0),
                        top: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                    },
                    ..default()
                },
                UiLife { min: i }, // 生命图标组件，标记它对应的生命数
            )
        })) ),
    ));
}

// 分数更新系统，监听Arena中的分数变化并更新UI文本
fn hud_score_system(arena: Res<Arena>, mut query: Query<&mut Text, With<UiScore>>) {
    if arena.is_changed() {
        for mut text in query.iter_mut() {
            **text = format!("{}", arena.score); // 更新文本显示最新分数
        }
    }
}

// 生命值更新系统，根据玩家飞船生命动态显示/隐藏生命图标
fn hud_life_system(
    mut commands: Commands,
    ship_query: Query<&Ship, Changed<Ship>>, // 监听飞船组件变化
    mut uilife_query: Query<(Entity, &UiLife)>, // 查询所有生命图标实体和组件
) {
    let mut life = 0;
    for ship in ship_query.iter() {
        if ship.player_id == 1 {
            life = ship.life; // 获取玩家1当前生命值
        }
    }
    for (entity, uilife) in uilife_query.iter_mut() {
        // 如果当前生命数 >= 该图标标记的最小生命数，显示图标，否则隐藏
        commands.entity(entity).insert(if life >= uilife.min {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}
