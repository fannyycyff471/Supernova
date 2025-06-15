use crate::prelude::*;

// 主状态枚举，区分游戏的不同“场景”或“阶段”
// 包含初始化、菜单、游戏和结尾界面
#[derive(States, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub enum AppState {
    #[default]
    Setup,   // 初始状态，游戏启动时默认状态
    Menu,    // 菜单界面状态
    Game,    // 游戏进行中状态
    Credits, // 制作人员名单界面状态
}

// 游戏主状态下的子状态枚举，进一步细化游戏内部流程
#[derive(SubStates, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
// 指定该子状态是 AppState::Game 的子状态
#[source(AppState=AppState::Game)]
pub enum GameState {
    #[default]
    Setup,   // 游戏准备阶段，如加载资源等
    Running, // 游戏实际运行中
    Paused,  // 游戏暂停状态
    Over,    // 游戏结束状态
}

// 管理状态切换的插件
pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        // 初始化主状态（AppState）
        app.init_state::<AppState>();
        // 使主状态下的实体跟随状态自动启用/禁用（状态范围内实体管理）
        app.enable_state_scoped_entities::<AppState>();
        // 添加游戏子状态（GameState）
        app.add_sub_state::<GameState>();
        // 同样启用子状态范围内实体管理
        app.enable_state_scoped_entities::<GameState>();

        // 注册状态切换的系统，带条件判断
        app.add_systems(
            Update,
            (
                // 当主状态处于 Setup 时，自动切换到 Menu
                transition_app_setup_to_menu.run_if(in_state(AppState::Setup)),
                // 当游戏子状态处于 Setup 时，自动切换到 Running
                transition_game_setup_to_running.run_if(in_state(GameState::Setup)),
            ),
        );
    }
}

// 从主状态的 Setup 自动切换到 Menu
fn transition_app_setup_to_menu(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::Menu);
}

// 从游戏子状态的 Setup 自动切换到 Running
fn transition_game_setup_to_running(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Running);
}
