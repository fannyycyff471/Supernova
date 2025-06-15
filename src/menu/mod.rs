use crate::prelude::*;

// 引入菜单相关模块
mod handler;
mod interaction;

// 暴露 handler 和 interaction 供外部使用
pub use handler::*;
pub use interaction::*;

// 定义菜单交互相关的用户动作
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum MenuAction {
    MenuUp,          // 菜单项向上移动
    MenuDown,        // 菜单项向下移动
    Accept,          // 确认选择（如 Enter）
    PauseUnpause,    // 暂停或取消暂停（Esc）
}

// 菜单插件结构体
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // 游戏初始化阶段：设置输入映射等
            .add_systems(OnEnter(AppState::Setup), setup)
            
            // 进入主菜单状态时生成主菜单
            .add_systems(OnEnter(AppState::Menu), spawn_main_menu)
            
            // 进入制作人员页面时生成 credits 菜单
            .add_systems(OnEnter(AppState::Credits), spawn_credits_menu)
            
            // 进入暂停状态时生成暂停菜单
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            
            // 进入游戏结束状态时生成游戏结束菜单
            .add_systems(OnEnter(GameState::Over), spawn_gameover_menu)
            
            // 每帧运行以下系统
            .add_systems(
                Update,
                (
                    main_menu_input_system,   // 处理主菜单的输入
                    menu_selection_system,    // 处理上下选择项移动
                    menu_blink_system,        // 控制文字闪烁效果
                ),
            )
            
            // 游戏状态下运行游戏菜单输入系统（仅限 AppState::Game）
            .add_systems(
                Update,
                game_menu_input_system.run_if(in_state(AppState::Game)),
            );
    }
}

// 设置菜单输入映射及初始资源
fn setup(mut commands: Commands) {
    let input_map = InputMap::<MenuAction>::new([
        (MenuAction::Accept, KeyCode::Enter),
        (MenuAction::PauseUnpause, KeyCode::Escape),
        (MenuAction::MenuUp, KeyCode::KeyW),
        (MenuAction::MenuUp, KeyCode::ArrowUp),
        (MenuAction::MenuDown, KeyCode::KeyS),
        (MenuAction::MenuDown, KeyCode::ArrowDown),
    ]);

    // 插入输入映射资源
    commands.insert_resource(input_map);
    // 插入动作状态资源（按键是否按下等）
    commands.insert_resource(ActionState::<MenuAction>::default());
}

// 生成主菜单界面
fn spawn_main_menu(mut commands: Commands, assets: ResMut<UiAssets>) {
    let entity = MenuHandler {
        main_text: "Supernova".into(),                       // 菜单标题文字
        main_text_color: Color::srgb(0.0, 0.7, 0.7),        // 标题颜色
        main_text_blink: false,                             // 是否闪烁
        selected_id: 0,                                     // 默认选中第一个选项
        entries: vec!["Play".into(), "Credits".into(), "Exit".into()],
    }
    .spawn(&mut commands, assets.font.clone());

    // 为菜单绑定状态作用域
    commands.entity(entity).insert(StateScoped(AppState::Menu));
}

// 生成游戏结束菜单
fn spawn_gameover_menu(mut commands: Commands, assets: ResMut<UiAssets>) {
    let entity = MenuHandler {
        main_text: "Game Over".into(),
        main_text_color: Color::srgb_u8(0xAA, 0x22, 0x22),
        main_text_blink: false,
        selected_id: 0,
        entries: vec!["Menu".into(), "Exit".into()],
    }
    .spawn(&mut commands, assets.font.clone());

    commands.entity(entity).insert(StateScoped(GameState::Over));
}

// 生成暂停菜单
fn spawn_pause_menu(mut commands: Commands, assets: ResMut<UiAssets>) {
    let entity = MenuHandler {
        main_text: "Pause".into(),
        main_text_color: Color::srgb_u8(0xF8, 0xE4, 0x73),
        main_text_blink: true,
        selected_id: 0,
        entries: vec!["Resume".into(), "Menu".into(), "Exit".into()],
    }
    .spawn(&mut commands, assets.font.clone());

    commands.entity(entity).insert(StateScoped(GameState::Paused));
}

// 生成 credits（制作人员名单）菜单
fn spawn_credits_menu(mut commands: Commands, assets: ResMut<UiAssets>) {
    let entity = MenuHandler {
        main_text: "".into(),                               // 没有大标题
        main_text_color: Color::srgb(0.0, 0.7, 0.7),
        main_text_blink: false,
        selected_id: 0,
        entries: vec!["Menu".into(), "Exit".into()],
    }
    .spawn(&mut commands, assets.font.clone());

    commands.entity(entity).insert(StateScoped(AppState::Credits));

    // 显示详细的开发人员与资源信息
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(70.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(AppState::Credits),
        children![
            (
                Text::new("Code"),
                TextFont {
                    font: assets.font.clone(),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.7, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                },
            ),
            (
                Text::default(),
                Node {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                children![
                    (
                        TextSpan::new(" Group CDLW "),        //小组成员首字母
                        TextFont {
                            font: assets.font_fira.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        TextSpan::new("(https://github.com/fannyycyff471)"),
                        TextFont {
                            font: assets.font_fira.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    )
                ]
            ),
            (
                Text::new("Assets"),                     //素材提供
                Node {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                TextFont {
                    font: assets.font.clone(),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.7, 0.7)),
            ),
            (
                Text::default(),
                Node {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                children![
                    (
                        TextSpan::new("Kenney Vleugels "),
                        TextFont {
                            font: assets.font_fira.clone(),
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        TextSpan::new("(www.kenney.nl)"),
                        TextFont {
                            font: assets.font_fira.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    )
                ]
            ),
        ],
    ));
}
